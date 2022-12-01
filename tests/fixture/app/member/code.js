import ServerComponent from "./ServerComponent";
import Client from "./Client";

export default function Page() {
  const rest = {};
  const date = new Date();

  return (
    <>
      <ServerComponent date={date} />
      <Client.Component date={date} {...rest} data-superjson />
    </>
  );
}
