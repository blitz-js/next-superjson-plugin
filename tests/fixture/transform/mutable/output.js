import { withSuperJSONPage as _withSuperJSONPage } from "next-superjson-plugin/tools";
import { withSuperJSONProps as _withSuperJSONProps } from "next-superjson-plugin/tools";
let foo = 1;
foo = 2;
export { foo as getServerSideProps };
foo = _withSuperJSONProps(() => {}, ["smth"]);
export default _withSuperJSONPage(() => {});
