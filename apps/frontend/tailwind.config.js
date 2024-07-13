/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/components/**/*.{js,vue,ts}",
    "./src/layouts/**/*.vue",
    "./src/pages/**/*.vue",
    "./src/plugins/**/*.{js,ts}",
    "./src/app.vue",
    "./src/error.vue",
    // monorepo - TODO: migrate this to its own package
    "../../packages/**/*.{js,vue,ts}",
  ],
  theme: {
    extend: {
      colors: {
        icon: "var(--color-icon)",
        // Text
        primary: "var(--color-text)",
        contrast: "var(--color-contrast)",
        secondary: "var(--color-secondary)",
        inactive: "var(--color-text-inactive)",
        dark: "var(--color-text-dark)",
        inverted: "var(--color-text-inverted)",
        heading: "var(--color-heading)",
        bg: {
          DEFAULT: "var(--color-bg)",
          red: "var(--color-red-bg)",
          raised: "var(--color-raised-bg)",
        },
        divider: {
          DEFAULT: "var(--color-divider)",
          dark: "var(--color-divider-dark)",
        },
        brand: {
          DEFAULT: "var(--color-brand)",
          green: "var(--color-brand-green)",
          highlight: "var(--color-brand-highlight)",
          shadow: "var(--color-brand-shadow)",
          inverted: "var(--color-brand-inverted)",
        },
        tabUnderlineHovered: "var(--tab-underline-hovered)",
        button: {
          bg: "var(--color-button-bg)",
          text: "var(--color-button-text)",
          bgHover: "var(--color-button-bg-hover)",
          textHover: "var(--color-button-text-hover)",
          bgActive: "var(--color-button-bg-active)",
          textActive: "var(--color-button-text-active)",
        },
        toggleHandle: "var(--color-toggle-handle)",
        dropdown: {
          bg: "var(--color-dropdown-bg)",
          text: "var(--color-dropdown-text)",
        },
        tooltip: {
          bg: "var(--color-tooltip-bg)",
          text: "var(--color-tooltip-text)",
        },
        code: {
          bg: "var(--color-code-bg)",
          text: "var(--color-code-text)",
        },
        kbdShadow: "var(--color-kbd-shadow)",
        ad: {
          DEFAULT: "var(--color-ad)",
          raised: "var(--color-ad-raised)",
          contrast: "var(--color-ad-contrast)",
          highlight: "var(--color-ad-highlight)",
        },
        greyLink: {
          DEFAULT: "var(--color-grey-link)",
          hover: "var(--color-grey-link-hover)",
          active: "var(--color-grey-link-active)",
        },
        link: {
          DEFAULT: "var(--color-link)",
          hover: "var(--color-link-hover)",
          active: "var(--color-link-active)",
        },
        warning: {
          bg: "var(--color-warning-bg)",
          text: "var(--color-warning-text)",
          banner: {
            text: "var(--color-warning-banner-text)",
            bg: "var(--color-warning-banner-bg)",
            side: "var(--color-warning-banner-side)",
          },
        },
        infoBanner: {
          text: "var(--color-info-banner-text)",
          bg: "var(--color-info-banner-bg)",
          side: "var(--color-info-banner-side)",
        },
        blockQuote: "var(--color-block-quote)",
        headerUnderline: "var(--color-header-underline)",
        hr: "var(--color-hr)",
        table: {
          border: "var(--color-table-border)",
          alternateRow: " var(--color-table-alternate-row)",
        },
      },
      backgroundImage: {
        mazeBg: "var(--landing-maze-bg)",
        mazeGradientBg: "var(--landing-maze-gradient-bg)",
        landing: {
          mazeOuterBg: "var(--landing-maze-outer-bg)",
          colorHeading: "var(--landing-color-heading)",
          colorSubheading: "var(--landing-color-subheading)",
          transitionGradientStart: "var(--landing-transition-gradient-start)",
          transitionGradientEnd: "var(--landing-transition-gradient-end)",
          hoverCardGradient: "var(--landing-hover-card-gradient)",
          borderGradient: "var(--landing-border-gradient)",
          borderColor: "var(--landing-border-color)",
          creatorGradient: "var(--landing-creator-gradient)",
          blobGradient: "var(--landing-blob-gradient)",
          cardBg: "var(--landing-card-bg)",
          blueLabel: "var(--landing-blue-label)",
          blueLabelBg: "var(--landing-blue-label-bg)",
          greenLabel: "var(--landing-green-label)",
          greenLabelBg: "var(--landing-green-label-bg)",
          rawBg: "var(--landing-raw-bg)",
        },
      },
    },
  },
  plugins: [],
};
