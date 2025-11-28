import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "FX eTrading Platform",
  description: "Ultra-Low-Latency FX eTrading Platform",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

