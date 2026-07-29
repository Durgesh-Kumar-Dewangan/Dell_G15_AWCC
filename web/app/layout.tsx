import type { Metadata, Viewport } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Dell G15 AWCC - Fan Control Center",
  description: "Advanced thermal monitoring and fan control for Dell G-Series laptops with real-time metrics and intelligent profiles",
};

export const viewport: Viewport = {
  themeColor: "#0066cc",
  userScalable: false,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="bg-background">
      <body className={`${inter.className} antialiased min-h-screen`}>
        {children}
      </body>
    </html>
  );
}
