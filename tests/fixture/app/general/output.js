import { SuperJSONComponent } from "next-superjson-plugin/tools";
import ServerComponent from "./ServerComponent";
import ClientComponent from "./ClientComponent";

export default function Page() {
  const rest = {};
  const date = new Date();

  return <>
      <ServerComponent date={date} />
      <SuperJSONComponent date={date} {...rest} _component={ClientComponent} />
    </>;
}
