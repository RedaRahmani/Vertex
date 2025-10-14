export type Fixture = {
  rpcUrl: string;
  programId: string;
  quoteMint: string;
  cpPool: string;
  policy: string;
  cpPosition: string;
  creatorAta: string;
  treasuryAta: string;
};

export const LOCAL_FIXTURE: Fixture = {
  "rpcUrl": "http://127.0.0.1:8899",
  "programId": "B4yaCkpGZB9Xnm2ZRcj9k1stdkXzkCJdbU26EWj8h7Dc",
  "quoteMint": "GEW4P2p4dx15LpGcdbvQcehS9URiDDDTM6mK8XtWZCVQ",
  "cpPool": "Snfmy76y3EiLoVUV6sYHXVoBUNz9HmQ3K3tezv9cPbq",
  "policy": "FiTABodxkrFmD8Vjiy9tEcQCwbNvgBRgqk4VbzjSbewU",
  "cpPosition": "9bdsv9udDXpiobcB4LbKJji3kagjmB4WgQhRjZSc6i7v",
  "creatorAta": "9DMae13YiUL5wrxJYXTHf4WM1msnxktRaGUkPju7JGYj",
  "treasuryAta": "3JYn7sGUSePuVQp4zKqiyjZk9edJSJY9AxExY6gubEFG"
};
