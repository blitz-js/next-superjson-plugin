export default function Page() {
  return <div>Page</div>;
}

export const getStaticProps = () => {
  return {
    props: {},
  };
}

export const getServerSideProps = () => {
  return {
    props: {},
  };
}

Page.getInitialProps = () => {
  return {};
}