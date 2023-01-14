import { serialize } from "next-superjson-plugin/tools";
import SuperJSONComponent from "next-superjson-plugin/client";
import ClientComponent from "./ClientComponent";
export default function Page() {
    const rest = {};
    const date = new Date();
    return <SuperJSONComponent props={serialize({
        date: date,
        ...rest
    })} component={ClientComponent}>

      <p >children</p>

    </SuperJSONComponent>;
}
