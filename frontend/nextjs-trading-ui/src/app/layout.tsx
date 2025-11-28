/**
 * Root layout for the application
 * 
 * @author Roberto de Souza <rabbittrix@hotmail.com>
 * @license Apache-2.0
 */

import type { Metadata } from "next";
import "./globals.css";
import MainLayout from "@/components/layout/MainLayout";

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
      <body>
        <MainLayout>{children}</MainLayout>
      </body>
    </html>
  );
}
