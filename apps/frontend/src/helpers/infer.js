import TOML from "@ltd/j-toml";
import JSZip from "jszip";
import yaml from "js-yaml";
import { satisfies } from "semver";

export const inferVersionInfo = async function (rawFile, project, gameVersions) {
  function versionType(number) {
    if (number.includes("alpha")) {
      return "alpha";
    } else if (
      number.includes("beta") ||
      number.match(/[^A-z](rc)[^A-z]/) || // includes `rc`
      number.match(/[^A-z](pre)[^A-z]/) // includes `pre`
    ) {
      return "beta";
    } else {
      return "release";
    }
  }

  function getGameVersionsMatchingSemverRange(range, gameVersions) {
    if (!range) {
      return [];
    }
    const ranges = Array.isArray(range) ? range : [range];
    return gameVersions.filter((version) => {
      const semverVersion = version.split(".").length === 2 ? `${version}.0` : version; // add patch version if missing (e.g. 1.16 -> 1.16.0)
      return ranges.some((v) => satisfies(semverVersion, v));
    });
  }

  function getGameVersionsMatchingMavenRange(range, gameVersions) {
    if (!range) {
      return [];
    }
    const ranges = [];

    while (range.startsWith("[") || range.startsWith("(")) {
      let index = range.indexOf(")");
      const index2 = range.indexOf("]");
      if (index === -1 || (index2 !== -1 && index2 < index)) {
        index = index2;
      }
      if (index === -1) break;
      ranges.push(range.substring(0, index + 1));
      range = range.substring(index + 1).trim();
      if (range.startsWith(",")) {
        range = range.substring(1).trim();
      }
    }

    if (range) {
      ranges.push(range);
    }

    const LESS_THAN_EQUAL = /^\(,(.*)]$/;
    const LESS_THAN = /^\(,(.*)\)$/;
    const EQUAL = /^\[(.*)]$/;
    const GREATER_THAN_EQUAL = /^\[(.*),\)$/;
    const GREATER_THAN = /^\((.*),\)$/;
    const BETWEEN = /^\((.*),(.*)\)$/;
    const BETWEEN_EQUAL = /^\[(.*),(.*)]$/;
    const BETWEEN_LESS_THAN_EQUAL = /^\((.*),(.*)]$/;
    const BETWEEN_GREATER_THAN_EQUAL = /^\[(.*),(.*)\)$/;

    const semverRanges = [];

    for (const range of ranges) {
      let result;
      if ((result = range.match(LESS_THAN_EQUAL))) {
        semverRanges.push(`<=${result[1]}`);
      } else if ((result = range.match(LESS_THAN))) {
        semverRanges.push(`<${result[1]}`);
      } else if ((result = range.match(EQUAL))) {
        semverRanges.push(`${result[1]}`);
      } else if ((result = range.match(GREATER_THAN_EQUAL))) {
        semverRanges.push(`>=${result[1]}`);
      } else if ((result = range.match(GREATER_THAN))) {
        semverRanges.push(`>${result[1]}`);
      } else if ((result = range.match(BETWEEN))) {
        semverRanges.push(`>${result[1]} <${result[2]}`);
      } else if ((result = range.match(BETWEEN_EQUAL))) {
        semverRanges.push(`>=${result[1]} <=${result[2]}`);
      } else if ((result = range.match(BETWEEN_LESS_THAN_EQUAL))) {
        semverRanges.push(`>${result[1]} <=${result[2]}`);
      } else if ((result = range.match(BETWEEN_GREATER_THAN_EQUAL))) {
        semverRanges.push(`>=${result[1]} <${result[2]}`);
      }
    }
    return getGameVersionsMatchingSemverRange(semverRanges, gameVersions);
  }

  const simplifiedGameVersions = gameVersions
    .filter((it) => it.version_type === "release")
    .map((it) => it.version);

  const inferFunctions = {
    // Forge 1.13+ and NeoForge
    "META-INF/mods.toml": async (file, zip) => {
      const metadata = TOML.parse(file, { joiner: "\n" });

      if (metadata.mods && metadata.mods.length > 0) {
        let versionNum = metadata.mods[0].version;

        // ${file.jarVersion} -> Implementation-Version from manifest
        const manifestFile = zip.file("META-INF/MANIFEST.MF");
        if (metadata.mods[0].version.includes("${file.jarVersion}") && manifestFile !== null) {
          const manifestText = await manifestFile.async("text");
          const regex = /Implementation-Version: (.*)$/m;
          const match = manifestText.match(regex);
          if (match) {
            versionNum = versionNum.replace("${file.jarVersion}", match[1]);
          }
        }

        let gameVersions = [];
        const mcDependencies = Object.values(metadata.dependencies)
          .flat()
          .filter((dependency) => dependency.modId === "minecraft");

        if (mcDependencies.length > 0) {
          gameVersions = getGameVersionsMatchingMavenRange(
            mcDependencies[0].versionRange,
            simplifiedGameVersions,
          );
        }

        const hasNeoForge =
          Object.values(metadata.dependencies)
            .flat()
            .filter((dependency) => dependency.modId === "neoforge").length > 0;

        const hasForge =
          Object.values(metadata.dependencies)
            .flat()
            .filter((dependency) => dependency.modId === "forge").length > 0;

        // Checks if game version is below 1.20.2 as NeoForge full split and id change was in 1.20.2
        const below1202 = getGameVersionsMatchingSemverRange("<=1.20.1", simplifiedGameVersions);

        const isOlderThan1202 = below1202.some((r) => gameVersions.includes(r));

        const loaders = [];

        if (hasNeoForge) loaders.push("neoforge");
        if (hasForge || isOlderThan1202) loaders.push("forge");

        return {
          name: `${project.title} ${versionNum}`,
          version_number: versionNum,
          version_type: versionType(versionNum),
          loaders,
          game_versions: gameVersions,
        };
      } else {
        return {};
      }
    },
    // Old Forge
    "mcmod.info": (file) => {
      const metadata = JSON.parse(file);

      return {
        name: metadata.version ? `${project.title} ${metadata.version}` : "",
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        loaders: ["forge"],
        game_versions: simplifiedGameVersions.filter((version) =>
          version.startsWith(metadata.mcversion),
        ),
      };
    },
    // Fabric
    "fabric.mod.json": (file) => {
      const metadata = JSON.parse(file);

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        loaders: ["fabric"],
        version_type: versionType(metadata.version),
        game_versions: metadata.depends
          ? getGameVersionsMatchingSemverRange(metadata.depends.minecraft, simplifiedGameVersions)
          : [],
      };
    },
    // Quilt
    "quilt.mod.json": (file) => {
      const metadata = JSON.parse(file);

      return {
        name: `${project.title} ${metadata.quilt_loader.version}`,
        version_number: metadata.quilt_loader.version,
        loaders: ["quilt"],
        version_type: versionType(metadata.quilt_loader.version),
        game_versions: metadata.quilt_loader.depends
          ? getGameVersionsMatchingSemverRange(
              metadata.quilt_loader.depends.find((x) => x.id === "minecraft")
                ? metadata.quilt_loader.depends.find((x) => x.id === "minecraft").versions
                : [],
              simplifiedGameVersions,
            )
          : [],
      };
    },
    // Bukkit + Other Forks
    "plugin.yml": (file) => {
      const metadata = yaml.load(file);

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        // We don't know which fork of Bukkit users are using
        loaders: [],
        game_versions: gameVersions
          .filter(
            (x) => x.version.startsWith(metadata["api-version"]) && x.version_type === "release",
          )
          .map((x) => x.version),
      };
    },
    // Paper 1.19.3+
    "paper-plugin.yml": (file) => {
      const metadata = yaml.load(file);

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        loaders: ["paper"],
        game_versions: gameVersions
          .filter(
            (x) => x.version.startsWith(metadata["api-version"]) && x.version_type === "release",
          )
          .map((x) => x.version),
      };
    },
    // Bungeecord + Waterfall
    "bungee.yml": (file) => {
      const metadata = yaml.load(file);

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        loaders: ["bungeecord"],
      };
    },
    // Velocity
    "velocity-plugin.json": (file) => {
      const metadata = JSON.parse(file);

      return {
        name: `${project.title} ${metadata.version}`,
        version_number: metadata.version,
        version_type: versionType(metadata.version),
        loaders: ["velocity"],
      };
    },
    // Modpacks
    "modrinth.index.json": (file) => {
      const metadata = JSON.parse(file);

      const loaders = [];
      if ("forge" in metadata.dependencies) {
        loaders.push("forge");
      }
      if ("neoforge" in metadata.dependencies) {
        loaders.push("neoforge");
      }
      if ("fabric-loader" in metadata.dependencies) {
        loaders.push("fabric");
      }
      if ("quilt-loader" in metadata.dependencies) {
        loaders.push("quilt");
      }

      return {
        name: `${project.title} ${metadata.versionId}`,
        version_number: metadata.versionId,
        version_type: versionType(metadata.versionId),
        loaders,
        game_versions: gameVersions
          .filter((x) => x.version === metadata.dependencies.minecraft)
          .map((x) => x.version),
      };
    },
    // Resource Packs + Data Packs
    "pack.mcmeta": (file) => {
      const metadata = JSON.parse(file);

      function getRange(versionA, versionB) {
        const startingIndex = gameVersions.findIndex((x) => x.version === versionA);
        const endingIndex = gameVersions.findIndex((x) => x.version === versionB);

        const final = [];
        const filterOnlyRelease = gameVersions[startingIndex].version_type === "release";

        for (let i = startingIndex; i >= endingIndex; i--) {
          if (gameVersions[i].version_type === "release" || !filterOnlyRelease) {
            final.push(gameVersions[i].version);
          }
        }

        return final;
      }

      const loaders = [];
      let newGameVersions = [];

      if (project.actualProjectType === "mod") {
        loaders.push("datapack");

        switch (metadata.pack.pack_format) {
          case 4:
            newGameVersions = getRange("1.13", "1.14.4");
            break;
          case 5:
            newGameVersions = getRange("1.15", "1.16.1");
            break;
          case 6:
            newGameVersions = getRange("1.16.2", "1.16.5");
            break;
          case 7:
            newGameVersions = getRange("1.17", "1.17.1");
            break;
          case 8:
            newGameVersions = getRange("1.18", "1.18.1");
            break;
          case 9:
            newGameVersions.push("1.18.2");
            break;
          case 10:
            newGameVersions = getRange("1.19", "1.19.3");
            break;
          case 11:
            newGameVersions = getRange("23w03a", "23w05a");
            break;
          case 12:
            newGameVersions.push("1.19.4");
            break;
          default:
        }
      }

      if (project.actualProjectType === "resourcepack") {
        loaders.push("minecraft");

        switch (metadata.pack.pack_format) {
          case 1:
            newGameVersions = getRange("1.6.1", "1.8.9");
            break;
          case 2:
            newGameVersions = getRange("1.9", "1.10.2");
            break;
          case 3:
            newGameVersions = getRange("1.11", "1.12.2");
            break;
          case 4:
            newGameVersions = getRange("1.13", "1.14.4");
            break;
          case 5:
            newGameVersions = getRange("1.15", "1.16.1");
            break;
          case 6:
            newGameVersions = getRange("1.16.2", "1.16.5");
            break;
          case 7:
            newGameVersions = getRange("1.17", "1.17.1");
            break;
          case 8:
            newGameVersions = getRange("1.18", "1.18.2");
            break;
          case 9:
            newGameVersions = getRange("1.19", "1.19.2");
            break;
          case 11:
            newGameVersions = getRange("22w42a", "22w44a");
            break;
          case 12:
            newGameVersions.push("1.19.3");
            break;
          case 13:
            newGameVersions.push("1.19.4");
            break;
          default:
        }
      }

      return {
        loaders,
        game_versions: newGameVersions,
      };
    },
  };

  const zipReader = new JSZip();

  const zip = await zipReader.loadAsync(rawFile);

  for (const fileName in inferFunctions) {
    const file = zip.file(fileName);

    if (file !== null) {
      const text = await file.async("text");
      return inferFunctions[fileName](text, zip);
    }
  }
};
