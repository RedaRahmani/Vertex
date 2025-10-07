export default function Page() {
  return (
    <main>
      <h2 className="text-lg font-medium mb-2">Dashboard</h2>
      <p className="text-sm text-gray-600">Connect your wallet in the pages below to interact with the fee router. This demo intentionally keeps UI minimal.</p>
      <ul className="list-disc pl-6 mt-4 text-sm space-y-1">
        <li>Initialize a Policy bound to a Meteora DLMM v2 pool</li>
        <li>Initialize the Honorary Position</li>
        <li>Run the Daily Crank and observe events</li>
      </ul>
    </main>
  );
}

