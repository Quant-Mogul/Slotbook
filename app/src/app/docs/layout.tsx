import Link from "next/link";
import { BrandMark } from "@/components/BrandMark";
import { DocsNav } from "@/components/DocsNav";
import { ThemeToggle } from "@/components/ThemeToggle";

export default function DocsLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="docs-shell">
      <header className="docs-header">
        <Link className="brand" href="/">
          <BrandMark />
          <span>SLOTBOOK / DOCS</span>
        </Link>
        <div className="docs-header-actions">
          <ThemeToggle />
          <Link href="/">Back to protocol ↗</Link>
        </div>
      </header>
      <div className="docs-layout">
        <aside className="docs-sidebar">
          <p className="eyebrow">DOCUMENTATION</p>
          <DocsNav />
          <p className="sidebar-note">
            Generated from
            <br />
            <code>programs/slotbook</code>
          </p>
        </aside>
        <article className="docs-content">{children}</article>
      </div>
      <footer className="docs-footer">
        <div className="pattern-dots docs-footer-dots" aria-hidden="true" />
        <div className="pattern-stripes docs-footer-stripes" aria-hidden="true" />
        <span>© 2026 SLOTBOOK / PROTOCOL REFERENCE</span>
        <span>GENERATED FROM PROGRAM SOURCE</span>
        <Link href="/">BACK TO PROTOCOL ↗</Link>
      </footer>
    </div>
  );
}
