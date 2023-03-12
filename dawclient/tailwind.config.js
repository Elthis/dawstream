module.exports = {
    mode: "all",
    content: [
        "./src/**/*.rs",
        "./index.html",
        "./src/**/*.html",
        "./src/**/*.css",
    ],
    theme: {
        extend: {
            gridTemplateColumns: {
              '24': 'repeat(24, minmax(0, 1fr))',
            }
        }
    },
    variants: {},
    plugins: [],
}