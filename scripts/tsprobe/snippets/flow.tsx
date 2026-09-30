// Flow sample (analysis-track §5.3, round 3) under the TSX grammar:
// the TypeScript table is shared, so every statement shape below must
// parse to the kinds the TypeScript sample shows, and JSX reads its
// identifiers through the general rule.

import * as React from "react";

type Props = { label: string; count: number };

function Badge({ label, count }: Props): JSX.Element {
  return (
    <span className="badge">
      {label}: {count}
    </span>
  );
}

export function Panel(props: Props): JSX.Element {
  let clicks = 0;
  const Item = Badge;
  const onClick = () => {
    clicks += 1;
  };
  if (props.count > 3) {
    return <Item label={props.label} count={clicks} />;
  } else if (props.count < 0) {
    throw new Error("negative");
  }
  for (let i = 0; i < props.count; i++) {
    if (i === 1) continue;
    clicks = clicks + i;
  }
  switch (props.label) {
    case "a":
      clicks++;
    default:
      clicks--;
  }
  try {
    clicks = JSON.parse(props.label);
  } catch (err) {
    clicks = 0;
  } finally {
    clicks += 0;
  }
  const rows = [1, 2, 3].map((n) => <li key={n}>{n + clicks}</li>);
  return (
    <div onClick={onClick}>
      {props.count > 0 && <Badge {...props} />}
      <ul>{rows}</ul>
    </div>
  );
}
