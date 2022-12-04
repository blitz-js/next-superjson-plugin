import { serialize } from "next-superjson-plugin/tools";
import SuperJSONComponent from "next-superjson-plugin/client";
import ServerComponent from "./ServerComponent";
import ClientComponent from "./ClientComponent";

export default function Page() {
  const rest = {};
  const date = new Date();

  return <>
      <ServerComponent date={date} />
      <SuperJSONComponent
        props={serialize({
          date: date,
          ...rest,
        })}
        component={ClientComponent}
      />
    </>;
}
