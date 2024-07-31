import withNuxt from "./.nuxt/eslint.config.mjs";

export default withNuxt().append({
  name: "modrinth/typescript",
  rules: {
    // *Generally* unhelpful, still nice to check from time to time!
    "@typescript-eslint/no-explicit-any": "off",
    // Conflicts with Prettier
    "vue/html-self-closing": "off",
    // Prone to breaking
    "no-undef": "off",
  },
});
