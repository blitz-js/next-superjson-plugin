import { withSuperJSONPage as _withSuperJSONPage } from "next-superjson-plugin/tools";
import { withSuperJSONInitProps as _withSuperJSONInitProps } from "next-superjson-plugin/tools";
import React from 'react'

class Page extends React.Component {
  static getInitialProps = _withSuperJSONInitProps(async function(ctx) {
    const res = await fetch('https://api.github.com/repos/vercel/next.js')
    const json = await res.json()
    return { stars: json.stargazers_count }
  }, ["smth"])

  render() {
    return <div>Next stars: {this.props.stars}</div>
  }
}

export default _withSuperJSONPage(Page)