import { withSuperJSONPage as _withSuperJSONPage } from "next-superjson-plugin/tools";
import { withSuperJSONProps as _withSuperJSONProps } from "next-superjson-plugin/tools";
import { foo as _NEXT_SUPERJSON_IMPORTED_PROPS, default as Page } from 'source';
const _NEXT_SUPERJSON_SSG_PROPS = _withSuperJSONProps(_NEXT_SUPERJSON_IMPORTED_PROPS, [
    "smth"
]);
export { _NEXT_SUPERJSON_SSG_PROPS as getServerSideProps };
export default _withSuperJSONPage(Page);