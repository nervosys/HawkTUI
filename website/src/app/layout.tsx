import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Louie \u2014 TUI Framework for Agentic AI",
  description:
    "An agentic-first terminal UI framework in Rust with complete ontology for agent discoverability.",
  openGraph: {
    title: "Louie \u2014 TUI Framework for Agentic AI",
    description:
      "An agentic-first terminal UI framework in Rust with complete ontology for agent discoverability.",
    url: "https://nervosys.ai/louie",
    siteName: "Louie",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;700&display=swap" rel="stylesheet" />
      </head>
      <body>{children}</body>
    </html>
  );
}
