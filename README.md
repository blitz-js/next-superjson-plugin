# next-superjson-plugin


next-superjson-plugin is a [SWC](https://swc.rs) plugin of `SuperJSON` for `Next.js (>=12.2.0)`

It supports transforming `getServerSideProps` & `getStaticProps` into [SuperJSON](https://github.com/blitz-js/superjson) functions

So that makes available to use complex objects(**Date, Map, Set..**) in props of pre-rendered pages

## Usage

Install packages first:

```
npm install superjson next-superjson-plugin
```

or using Yarn:

```
yarn add superjson next-superjson-plugin
```

Then modify `next.config.js` in the root directory of your Next.js project:

```js
// next.config.js
module.exports = {
  experimental: {
    swcPlugins: [
      [
        'next-superjson-plugin',
        {
          excluded: [],
        },
      ],
    ],
  },
}
```

### Option
You can use the `exclude` option to exclude specific properties from serialization.
```js
{
  excluded: ["someProp"],
},
```

## Contributing

[Leave an issue](https://github.com/orionmiz/next-superjson-plugin/issues)

## Special Thanks
- [kdy1](https://github.com/kdy1) (Main creator of swc project)