import { useAutoRef, type AutoRef } from "./auto-ref.ts";

const safeTags = new Map<string, string>();

function safeTagFor(locale: string) {
  let safeTag = safeTags.get(locale);
  if (safeTag == null) {
    safeTag = new Intl.Locale(locale).baseName;
    safeTags.set(locale, safeTag);
  }
  return safeTag;
}

type DisplayNamesWrapper = Intl.DisplayNames & {
  of(tag: string): string | undefined;
};

const displayNamesDicts = new Map<string, DisplayNamesWrapper>();

function getWrapperKey(locale: string, options: Intl.DisplayNamesOptions) {
  return JSON.stringify({ ...options, locale });
}

export function createDisplayNames(
  locale: string,
  options: Intl.DisplayNamesOptions = { type: "language" },
) {
  const wrapperKey = getWrapperKey(locale, options);
  let wrapper = displayNamesDicts.get(wrapperKey);

  if (wrapper == null) {
    const dict = new Intl.DisplayNames(locale, options);

    const badTags: string[] = [];

    wrapper = {
      resolvedOptions() {
        return dict.resolvedOptions();
      },
      of(tag: string) {
        let attempt = 0;

        lookupLoop: do {
          let lookup: string;
          switch (attempt) {
            case 0:
              lookup = tag;
              break;
            case 1:
              lookup = safeTagFor(tag);
              break;
            default:
              break lookupLoop;
          }

          if (badTags.includes(lookup)) continue;

          try {
            return dict.of(lookup);
          } catch (err) {
            console.warn(
              `Failed to get display name for ${lookup} using dictionary for ${
                this.resolvedOptions().locale
              }`,
            );
            badTags.push(lookup);
            continue;
          }
        } while (++attempt < 5);

        return undefined;
      },
    };

    displayNamesDicts.set(wrapperKey, wrapper);
  }

  return wrapper;
}

export function useDisplayNames(
  locale: AutoRef<string>,
  options?: AutoRef<Intl.DisplayNamesOptions | undefined>,
) {
  const $locale = useAutoRef(locale);
  const $options = useAutoRef(options);

  return computed(() => createDisplayNames($locale.value, $options.value));
}
