export default function Page() {
  return <div>Page</div>;
}

Page.getInitialProps = () => {
  return {};
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