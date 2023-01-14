import ClientComponent from "./ClientComponent";

export default function Page() {
  const rest = {};
  const date = new Date();

  return (
    <ClientComponent date={date} {...rest} data-superjson>
      <p>children</p>
    </ClientComponent>
  );
}
