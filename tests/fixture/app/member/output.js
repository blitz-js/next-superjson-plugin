import { SuperJSONComponent } from "next-superjson-plugin/tools";
import ServerComponent from "./ServerComponent";
import Client from "./Client";

export default function Page() {
  const rest = {};
  const date = new Date();

  return <>
      <ServerComponent date={date} />
      <SuperJSONComponent date={date} {...rest} _component={Client.Component} />
    </>;
}
