import ServerComponent from "./ServerComponent";
import ClientComponent from "./ClientComponent";

export default function Page() {
  const rest = {};
  const date = new Date();

  return (
    <>
      <ServerComponent date={date} />
      <ClientComponent date={date} {...rest} data-superjson />
    </>
  );
}
