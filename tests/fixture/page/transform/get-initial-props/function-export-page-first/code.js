export default function Page({ date }) {
  return <div>{date.getDate()}</div>
}

Page.getInitialProps = () => {
  return {
    date: new Date()
  }
}