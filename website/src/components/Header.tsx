"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const links = [
  { href: "/docs/getting-started", label: "Docs" },
  { href: "/docs/widgets", label: "Widgets" },
  { href: "/docs/agent-protocol", label: "Agent Protocol" },
  { href: "/docs/examples", label: "Examples" },
  { href: "https://github.com/nervosys/louie", label: "GitHub" },
];

export default function Header() {
  const pathname = usePathname();
  return (
    <header className="header">
      <Link href="/" className="logo">
        <span>[</span>LOUIE<span>]</span>
      </Link>
      <nav>
        {links.map((l) => (
          <Link
            key={l.href}
            href={l.href}
            className={pathname.startsWith(l.href) ? "active" : ""}
          >
            {l.label}
          </Link>
        ))}
      </nav>
    </header>
  );
}
