import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Slotbook — Verifiable distributions at every slot",
  description: "A disputable holder register for permissioned token distributions.",
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>{children}</body>
    </html>
  );
}
