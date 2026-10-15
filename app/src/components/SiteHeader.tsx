import Link from "next/link";
import { BrandMark } from "@/components/BrandMark";
import { ThemeToggle } from "@/components/ThemeToggle";

export function SiteHeader() {
  return (
    <header className="site-header">
      <Link className="brand" href="/" aria-label="Slotbook home">
        <BrandMark />
        <span>SLOTBOOK</span>
      </Link>
      <nav className="nav-links" aria-label="Primary navigation">
        <a href="#protocol">Protocol</a>
        <a href="#flow">How it works</a>
        <Link href="/docs">Docs</Link>
      </nav>
      <div className="header-actions">
        <span className="network-pill">
          <i /> DEVNET LIVE // V2.1
        </span>
        <ThemeToggle />
        <Link className="angular-button compact" href="/docs">
          Read the protocol <span>↗</span>
        </Link>
      </div>
    </header>
  );
}
