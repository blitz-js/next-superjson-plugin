"use client";

import { SuperJSONProps, withSuperJSONPage } from "./tools";
import * as React from "react";

export default function SuperJSONComponent<P>({
  component,
  props,
}: {
  component: React.ComponentType<P>;
  props: SuperJSONProps<P>;
}) {
  const WithSuperJSON = withSuperJSONPage(component);
  return <WithSuperJSON {...props} />;
}
