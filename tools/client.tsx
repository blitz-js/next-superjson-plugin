"use client";

import { SuperJSONProps, withSuperJSONPage } from "./tools";
import * as React from "react";

export default function SuperJSONComponent<P>({
  component,
  props,
  children
}: {
  component: React.ComponentType<P>;
  props: SuperJSONProps<P>;
  children?: React.ReactNode;
}) {
  const WithSuperJSON = withSuperJSONPage(component);
  return <WithSuperJSON {...props}>{children}</WithSuperJSON>;
}
