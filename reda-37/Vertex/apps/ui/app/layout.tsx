import Link from 'next/link';
import React from 'react';
import './globals.css';
import { EnvironmentPanel, EnvironmentProvider } from './environment';

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <EnvironmentProvider>
          <div className="mx-auto max-w-4xl p-6">
            <header className="mb-6 flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
              <h1 className="text-xl font-semibold">Vertex Fee Router</h1>
              <nav className="flex flex-wrap gap-3 text-sm underline">
                <Link href="/">Dashboard</Link>
                <Link href="/policy-setup">Policy Setup</Link>
                <Link href="/honorary-position">Honorary Position</Link>
                <Link href="/daily-crank">Daily Crank</Link>
              </nav>
            </header>
            <EnvironmentPanel />
            {children}
          </div>
        </EnvironmentProvider>
      </body>
    </html>
  );
}
