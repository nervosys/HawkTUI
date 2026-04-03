"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";

const sections = [
  {
    title: "Getting Started",
    links: [
      { href: "/docs/getting-started", label: "Introduction" },
      { href: "/docs/installation", label: "Installation" },
      { href: "/docs/quick-start", label: "Quick Start" },
      { href: "/docs/architecture", label: "Architecture" },
    ],
  },
  {
    title: "Core Concepts",
    links: [
      { href: "/docs/elm-architecture", label: "Elm Architecture" },
      { href: "/docs/ontology", label: "Ontology" },
      { href: "/docs/layout", label: "Layout" },
      { href: "/docs/events", label: "Events" },
      { href: "/docs/animation", label: "Animation" },
      { href: "/docs/focus-overlays", label: "Focus & Overlays" },
    ],
  },
  {
    title: "Widgets",
    links: [{ href: "/docs/widgets", label: "Widget Catalog" }],
  },
  {
    title: "Agent Integration",
    links: [
      { href: "/docs/agent-protocol", label: "Protocol Reference" },
      { href: "/docs/agent-integration", label: "Integration Guide" },
      { href: "/docs/headless-driver", label: "Headless Driver" },
    ],
  },
  {
    title: "More",
    links: [
      { href: "/docs/examples", label: "Examples" },
      { href: "/docs/security", label: "Security" },
      { href: "/docs/contributing", label: "Contributing" },
      { href: "/docs/changelog", label: "Changelog" },
    ],
  },
];

export default function Sidebar() {
  const pathname = usePathname();
  const [open, setOpen] = useState<Record<string, boolean>>(
    Object.fromEntries(sections.map((s) => [s.title, true]))
  );

  return (
    <aside className="sidebar">
      {sections.map((s) => (
        <div key={s.title} className="sidebar-section">
          <button onClick={() => setOpen((o) => ({ ...o, [s.title]: !o[s.title] }))}>
            {s.title}
            <span>{open[s.title] ? "\u25B4" : "\u25BE"}</span>
          </button>
          {open[s.title] && (
            <ul className="sidebar-links">
              {s.links.map((l) => (
                <li key={l.href}>
                  <Link
                    href={l.href}
                    className={pathname === l.href ? "active" : ""}
                  >
                    {l.label}
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </aside>
  );
}
