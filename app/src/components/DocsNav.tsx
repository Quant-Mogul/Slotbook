"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const groups = [
  {
    label: "START HERE",
    links: [
      ["Overview", "/docs"],
      ["Lifecycle", "/docs#lifecycle"],
      ["Core concepts", "/docs#concepts"],
    ],
  },
  { label: "PROTOCOL", links: [["Program methods", "/docs/program"]] },
  {
    label: "WORKFLOW",
    links: [
      ["01 / Declare", "/docs/workflow/declare"],
      ["02 / Commit root", "/docs/workflow/commit"],
      ["03 / Challenge", "/docs/workflow/challenge"],
      ["04 / Claim", "/docs/workflow/claim"],
    ],
  },
] as const;

export function DocsNav() {
  const pathname = usePathname();
  return (
    <nav>
      {groups.map((group) => (
        <div className="docs-nav-group" key={group.label}>
          <span className="docs-nav-label">{group.label}</span>
          {group.links.map(([label, href]) => {
            const path = href.split("#")[0];
            const active = pathname === path && (path !== "/docs" || !href.includes("#"));
            return (
              <Link
                className={active ? "active" : ""}
                aria-current={active ? "page" : undefined}
                href={href}
                key={href}
              >
                {label}
              </Link>
            );
          })}
        </div>
      ))}
    </nav>
  );
}
