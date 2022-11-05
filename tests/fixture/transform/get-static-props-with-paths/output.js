import { withSuperJSONPage as _withSuperJSONPage } from "next-superjson-plugin/tools";
import { withSuperJSONProps as _withSuperJSONProps } from "next-superjson-plugin/tools";
export const getStaticProps = _withSuperJSONProps(() => {}, ["smth"]);
export const getStaticPaths = () => {};
export default _withSuperJSONPage(() => {
  return <></>;
});
