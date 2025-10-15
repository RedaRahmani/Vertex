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
  "quoteMint": "GjN9SDWzXMFMShRCdQTf62UANFXUnFUQApBMY4Qzot4n",
  "cpPool": "D6RCRUzBtDYmsYf5RiJr1GJL9j6KFYnBFaYBhab7rGUs",
  "policy": "7S2BPVYjEVvC1dtizVB3sz2qxsmr18WwyYmCDEnhhAXZ",
  "cpPosition": "89pHhaNbz1JEChMiteqVoSbSTFUaujJJb1BkMhyvReE9",
  "creatorAta": "BJj5waiGfytPzzsH4WnQcwYtkvSmY6srv2yZv5nfEB2p",
  "treasuryAta": "BcxULpoo4zCG5BkQge3w7E8cACJZcGrfi3kyejjZeBBy"
};
