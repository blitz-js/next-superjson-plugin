import { withSuperJSONPage as _withSuperJSONPage } from "next-superjson-plugin/tools";
import { withSuperJSONProps as _withSuperJSONProps } from "next-superjson-plugin/tools";
export const getServerSideProps = _withSuperJSONProps(async () => {}, [
    "smth"
]);
export default _withSuperJSONPage(() => {
    return <></>;
});
