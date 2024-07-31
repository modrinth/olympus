import "@vintl/vintl";
import type { CompactNumber } from "@vintl/compact-number/dist/index.mjs";

declare global {
  namespace VueIntlController {
    interface MessageValueTypes {
      compactNumber: CompactNumber;
    }

    interface LocaleResources {
      "languages.json"?: Partial<Record<string, string>>;
    }

    interface LocaleMeta {
      displayName?: string;
      category?: string;
      searchTerms?: string;
    }
  }
}
