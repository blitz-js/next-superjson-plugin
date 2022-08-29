import { withSuperJSONPage as _withSuperJSONPage } from "next-superjson-plugin/tools";
import { withSuperJSONInitProps as _withSuperJSONInitProps } from "next-superjson-plugin/tools";

function Page({ date }) {
  return <div>{date.getDate()}</div>
}

Page.getInitialProps = _withSuperJSONInitProps(() => {
  return {
    date: new Date()
  }
}, ["smth"]);

export default _withSuperJSONPage(Page);