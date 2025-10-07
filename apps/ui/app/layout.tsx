import './globals.css';
import React from 'react';

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <div className="max-w-4xl mx-auto p-6">
          <header className="mb-6 flex items-center justify-between">
            <h1 className="text-xl font-semibold">Vertex Fee Router</h1>
            <nav className="space-x-3 text-sm">
              <a href="/" className="underline">Dashboard</a>
              <a href="/policy-setup" className="underline">Policy Setup</a>
              <a href="/honorary-position" className="underline">Honorary Position</a>
              <a href="/daily-crank" className="underline">Daily Crank</a>
            </nav>
          </header>
          {children}
        </div>
      </body>
    </html>
  );
}

