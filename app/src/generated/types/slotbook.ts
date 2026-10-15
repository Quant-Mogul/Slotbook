/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/slotbook.json`.
 */
export type Slotbook = {
  "address": "3ZaUF1Q6FH9HaD7kjMNVztaFtJxQY9U3Zk7CuR3wAqJp",
  "metadata": {
    "name": "slotbook",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "docs": [
    "Slotbook: a verifiable, disputable record-date holder register for permissioned",
    "SPL tokens. Use cases UC-1 to UC-14 and decisions D1 to D22 are in the scope lock;",
    "the snapshot format is docs/SNAPSHOT_SPEC.md."
  ],
  "instructions": [
    {
      "name": "challenge",
      "discriminator": [
        16,
        107,
        14,
        39,
        244,
        150,
        81,
        187
      ],
      "accounts": [
        {
          "name": "challenger",
          "writable": true,
          "signer": true
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          }
        },
        {
          "name": "challenge",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  97,
                  108,
                  108,
                  101,
                  110,
                  103,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "distribution.challengeCount",
                "account": "distribution"
              }
            ]
          }
        },
        {
          "name": "bondMint"
        },
        {
          "name": "bondVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "bondTokenProgram"
              },
              {
                "kind": "account",
                "path": "bondMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "challengerBondAccount",
          "writable": true
        },
        {
          "name": "bondTokenProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "owner",
          "type": "pubkey"
        },
        {
          "name": "claimedBalance",
          "type": "u64"
        }
      ]
    },
    {
      "name": "claim",
      "discriminator": [
        62,
        198,
        214,
        193,
        213,
        159,
        108,
        210
      ],
      "accounts": [
        {
          "name": "holder",
          "docs": [
            "Must be the leaf owner (D10)."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          }
        },
        {
          "name": "claimReceipt",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "holder"
              }
            ]
          }
        },
        {
          "name": "pending",
          "docs": [
            "Created in the handler only when the holder's account is not transferable,",
            "because Anchor cannot init on one branch."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  101,
                  110,
                  100,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "holder"
              }
            ]
          }
        },
        {
          "name": "paymentMint",
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "paymentTokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "holderPaymentAccount",
          "writable": true
        },
        {
          "name": "holderMintAccount",
          "docs": [
            "The holder's associated token account of the permissioned mint; read for freeze state.",
            "It must be the ATA: with any account of theirs, a holder with one frozen and one",
            "thawed account could pass the thawed one and skip Pending."
          ]
        },
        {
          "name": "paymentTokenProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "balance",
          "type": "u64"
        },
        {
          "name": "salt",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "proof",
          "type": {
            "vec": {
              "array": [
                "u8",
                32
              ]
            }
          }
        }
      ]
    },
    {
      "name": "closeDistribution",
      "discriminator": [
        238,
        70,
        219,
        176,
        69,
        243,
        141,
        230
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true,
          "relations": [
            "issuerConfig"
          ]
        },
        {
          "name": "issuerConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          }
        },
        {
          "name": "paymentMint",
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "paymentTokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "issuerPaymentAccount",
          "writable": true
        },
        {
          "name": "paymentTokenProgram"
        }
      ],
      "args": []
    },
    {
      "name": "commitRoot",
      "discriminator": [
        142,
        242,
        81,
        248,
        39,
        238,
        194,
        125
      ],
      "accounts": [
        {
          "name": "attestor",
          "writable": true,
          "signer": true
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          }
        },
        {
          "name": "attestorAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "attestor"
              }
            ]
          }
        },
        {
          "name": "commitment",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  109,
                  109,
                  105,
                  116,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "attestor"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "commitArgs"
            }
          }
        }
      ]
    },
    {
      "name": "declareDistribution",
      "discriminator": [
        29,
        50,
        99,
        125,
        40,
        91,
        172,
        195
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true,
          "relations": [
            "issuerConfig"
          ]
        },
        {
          "name": "issuerConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          }
        },
        {
          "name": "paymentMint"
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "issuerConfig.distributionCount",
                "account": "issuerConfig"
              }
            ]
          }
        },
        {
          "name": "vault",
          "docs": [
            "init_if_needed: anyone can create an ATA for any address, so a plain `init`",
            "could be blocked by someone creating this vault first."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "paymentTokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "issuerPaymentAccount",
          "writable": true
        },
        {
          "name": "paymentTokenProgram"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "total",
          "type": "u64"
        },
        {
          "name": "recordSlot",
          "type": "u64"
        },
        {
          "name": "saltSeedHash",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        }
      ]
    },
    {
      "name": "finalize",
      "discriminator": [
        171,
        61,
        218,
        56,
        127,
        115,
        12,
        217
      ],
      "accounts": [
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "distribution.issuerConfig",
                "account": "distribution"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "initIssuer",
      "discriminator": [
        186,
        77,
        49,
        118,
        203,
        47,
        30,
        79
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "mint",
          "docs": [
            "The permissioned Token-2022 mint being registered."
          ]
        },
        {
          "name": "bondMint",
          "docs": [
            "Bonds are paid in this mint (D9, devnet USDC)."
          ]
        },
        {
          "name": "issuerConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "bondVault",
          "docs": [
            "init_if_needed: anyone can create an ATA for any address, so a plain `init`",
            "could be blocked by someone creating this vault first."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "bondTokenProgram"
              },
              {
                "kind": "account",
                "path": "bondMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "bondTokenProgram"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "issuerParams"
            }
          }
        }
      ]
    },
    {
      "name": "registerAttestor",
      "discriminator": [
        68,
        201,
        86,
        244,
        51,
        113,
        15,
        94
      ],
      "accounts": [
        {
          "name": "attestor",
          "writable": true,
          "signer": true
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          }
        },
        {
          "name": "attestorAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "attestor"
              }
            ]
          }
        },
        {
          "name": "bondMint"
        },
        {
          "name": "bondVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "bondTokenProgram"
              },
              {
                "kind": "account",
                "path": "bondMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "attestorBondAccount",
          "writable": true
        },
        {
          "name": "bondTokenProgram"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "backend",
          "type": {
            "defined": {
              "name": "backend"
            }
          }
        }
      ]
    },
    {
      "name": "releaseAttestor",
      "discriminator": [
        158,
        67,
        176,
        176,
        4,
        182,
        206,
        13
      ],
      "accounts": [
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "distribution.issuerConfig",
                "account": "distribution"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          },
          "relations": [
            "commitment"
          ]
        },
        {
          "name": "attestorAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "distribution.issuerConfig",
                "account": "distribution"
              },
              {
                "kind": "account",
                "path": "commitment.attestor",
                "account": "commitment"
              }
            ]
          }
        },
        {
          "name": "commitment",
          "docs": [
            "Closes to the Attestor account, like a commitment voided in resolve_challenge,",
            "so every commitment's rent reaches the attestor the same way: at withdraw_bond."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  109,
                  109,
                  105,
                  116,
                  109,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "commitment.attestor",
                "account": "commitment"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "releasePending",
      "discriminator": [
        24,
        226,
        163,
        4,
        150,
        253,
        20,
        170
      ],
      "accounts": [
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          },
          "relations": [
            "pending"
          ]
        },
        {
          "name": "pending",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  101,
                  110,
                  100,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "pending.owner",
                "account": "pending"
              }
            ]
          }
        },
        {
          "name": "payer",
          "docs": [
            "Receives the Pending rent (D11)."
          ],
          "writable": true,
          "relations": [
            "pending"
          ]
        },
        {
          "name": "paymentMint",
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "paymentTokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "holderPaymentAccount",
          "writable": true
        },
        {
          "name": "holderMintAccount",
          "docs": [
            "The holder's associated token account (see Claim), so a thawed side account cannot",
            "release a payout held for a frozen holder."
          ]
        },
        {
          "name": "paymentTokenProgram"
        }
      ],
      "args": []
    },
    {
      "name": "resolveChallenge",
      "discriminator": [
        81,
        191,
        124,
        119,
        131,
        248,
        157,
        109
      ],
      "accounts": [
        {
          "name": "signer",
          "docs": [
            "The resolver, or any cranker for Ruling::Expired."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          }
        },
        {
          "name": "challenge",
          "docs": [
            "Present for an owner challenge; absent for a root conflict (D3)."
          ],
          "writable": true,
          "optional": true
        },
        {
          "name": "challenger",
          "docs": [
            "Receives the Challenge rent; must equal challenge.challenger."
          ],
          "writable": true,
          "optional": true
        },
        {
          "name": "bondMint"
        },
        {
          "name": "bondVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "bondTokenProgram"
              },
              {
                "kind": "account",
                "path": "bondMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "resolverBondAccount",
          "writable": true
        },
        {
          "name": "challengerBondAccount",
          "docs": [
            "Challenger's bond-mint account; owner checked in the handler."
          ],
          "writable": true,
          "optional": true
        },
        {
          "name": "issuerBondAccount",
          "writable": true
        },
        {
          "name": "bondTokenProgram"
        }
      ],
      "args": [
        {
          "name": "ruling",
          "type": {
            "defined": {
              "name": "ruling"
            }
          }
        }
      ]
    },
    {
      "name": "sweepPending",
      "discriminator": [
        4,
        68,
        92,
        154,
        177,
        126,
        175,
        214
      ],
      "accounts": [
        {
          "name": "authority",
          "signer": true,
          "relations": [
            "issuerConfig"
          ]
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          },
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "distribution",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  100,
                  105,
                  115,
                  116,
                  114,
                  105,
                  98,
                  117,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "distribution.id",
                "account": "distribution"
              }
            ]
          },
          "relations": [
            "pending"
          ]
        },
        {
          "name": "pending",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  101,
                  110,
                  100,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "pending.owner",
                "account": "pending"
              }
            ]
          }
        },
        {
          "name": "payer",
          "docs": [
            "Receives the Pending rent (D11)."
          ],
          "writable": true,
          "relations": [
            "pending"
          ]
        },
        {
          "name": "paymentMint",
          "relations": [
            "distribution"
          ]
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "distribution"
              },
              {
                "kind": "account",
                "path": "paymentTokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "issuerPaymentAccount",
          "writable": true
        },
        {
          "name": "paymentTokenProgram"
        }
      ],
      "args": []
    },
    {
      "name": "updateIssuer",
      "discriminator": [
        9,
        100,
        234,
        30,
        84,
        43,
        30,
        29
      ],
      "accounts": [
        {
          "name": "authority",
          "signer": true,
          "relations": [
            "issuerConfig"
          ]
        },
        {
          "name": "issuerConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": {
              "name": "issuerParams"
            }
          }
        }
      ]
    },
    {
      "name": "withdrawBond",
      "discriminator": [
        222,
        199,
        141,
        31,
        188,
        93,
        155,
        40
      ],
      "accounts": [
        {
          "name": "attestor",
          "writable": true,
          "signer": true
        },
        {
          "name": "issuerConfig",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  105,
                  115,
                  115,
                  117,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig.mint",
                "account": "issuerConfig"
              }
            ]
          }
        },
        {
          "name": "attestorAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  116,
                  116,
                  101,
                  115,
                  116,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "attestor"
              }
            ]
          }
        },
        {
          "name": "bondMint"
        },
        {
          "name": "bondVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "issuerConfig"
              },
              {
                "kind": "account",
                "path": "bondTokenProgram"
              },
              {
                "kind": "account",
                "path": "bondMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "attestorBondAccount",
          "writable": true
        },
        {
          "name": "bondTokenProgram"
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "attestor",
      "discriminator": [
        253,
        240,
        76,
        196,
        16,
        53,
        239,
        173
      ]
    },
    {
      "name": "challenge",
      "discriminator": [
        119,
        250,
        161,
        121,
        119,
        81,
        22,
        208
      ]
    },
    {
      "name": "claimReceipt",
      "discriminator": [
        223,
        233,
        11,
        229,
        124,
        165,
        207,
        28
      ]
    },
    {
      "name": "commitment",
      "discriminator": [
        61,
        112,
        129,
        128,
        24,
        147,
        77,
        87
      ]
    },
    {
      "name": "distribution",
      "discriminator": [
        176,
        85,
        17,
        11,
        13,
        194,
        18,
        1
      ]
    },
    {
      "name": "issuerConfig",
      "discriminator": [
        238,
        244,
        71,
        221,
        254,
        169,
        247,
        237
      ]
    },
    {
      "name": "pending",
      "discriminator": [
        36,
        180,
        147,
        170,
        72,
        175,
        173,
        126
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "notImplemented",
      "msg": "Instruction not implemented yet"
    },
    {
      "code": 6001,
      "name": "notMintAuthority",
      "msg": "Signer is not the mint's mint authority, or it is unset"
    },
    {
      "code": 6002,
      "name": "freezeAuthorityNotTokenAcl",
      "msg": "Mint freeze authority is not the Token ACL MintConfig PDA"
    },
    {
      "code": 6003,
      "name": "missingDefaultAccountState",
      "msg": "Mint must carry DefaultAccountState"
    },
    {
      "code": 6004,
      "name": "defaultStateNotFrozen",
      "msg": "Mint DefaultAccountState must be Frozen"
    },
    {
      "code": 6005,
      "name": "confidentialTransferMint",
      "msg": "Mints with the confidential-transfer extension are not supported"
    },
    {
      "code": 6006,
      "name": "invalidParams",
      "msg": "Invalid issuer parameters"
    },
    {
      "code": 6007,
      "name": "tooManyAttestors",
      "msg": "Too many attestors"
    },
    {
      "code": 6008,
      "name": "resolverConflict",
      "msg": "Resolver must be neither the authority nor an attestor"
    },
    {
      "code": 6009,
      "name": "resolverFeeTooHigh",
      "msg": "resolver_fee must be below both bonds"
    },
    {
      "code": 6010,
      "name": "activeDistributions",
      "msg": "Distributions are still active"
    },
    {
      "code": 6011,
      "name": "notAuthority",
      "msg": "Signer is not the issuer authority"
    },
    {
      "code": 6012,
      "name": "recordSlotTooSoon",
      "msg": "Record slot is not far enough in the future"
    },
    {
      "code": 6013,
      "name": "unsupportedPaymentMint",
      "msg": "Payment mint has a disallowed extension"
    },
    {
      "code": 6014,
      "name": "unsupportedBondMint",
      "msg": "Bond mint has a disallowed extension"
    },
    {
      "code": 6015,
      "name": "invalidState",
      "msg": "Distribution is not in the required state"
    },
    {
      "code": 6016,
      "name": "cannotClose",
      "msg": "Distribution cannot be closed yet"
    },
    {
      "code": 6017,
      "name": "notAnAttestor",
      "msg": "Signer is not in the issuer's attestor set"
    },
    {
      "code": 6018,
      "name": "finalityMarginNotPassed",
      "msg": "Finality margin after the record slot has not passed"
    },
    {
      "code": 6019,
      "name": "insufficientBond",
      "msg": "Attestor bond does not cover another commitment"
    },
    {
      "code": 6020,
      "name": "rootMismatch",
      "msg": "Root or register_total does not match the final commitment"
    },
    {
      "code": 6021,
      "name": "unsupportedSpecVersion",
      "msg": "Spec version is not supported"
    },
    {
      "code": 6022,
      "name": "liveCommitments",
      "msg": "Attestor still has live commitments"
    },
    {
      "code": 6023,
      "name": "emptyRegister",
      "msg": "Register is empty; nothing to commit (spec 3.7)"
    },
    {
      "code": 6024,
      "name": "windowClosed",
      "msg": "Challenge window is closed"
    },
    {
      "code": 6025,
      "name": "windowNotElapsed",
      "msg": "Challenge window has not elapsed"
    },
    {
      "code": 6026,
      "name": "notResolver",
      "msg": "Signer is not the resolver"
    },
    {
      "code": 6027,
      "name": "timeoutNotReached",
      "msg": "Resolve timeout has not been reached"
    },
    {
      "code": 6028,
      "name": "commitmentSetMismatch",
      "msg": "Remaining accounts must be every live commitment and its attestor"
    },
    {
      "code": 6029,
      "name": "wrongChallenge",
      "msg": "Challenge account does not match the open challenge"
    },
    {
      "code": 6030,
      "name": "invalidRuling",
      "msg": "Ruling does not fit this kind of dispute"
    },
    {
      "code": 6031,
      "name": "claimExpired",
      "msg": "Claim period has expired"
    },
    {
      "code": 6032,
      "name": "proofTooDeep",
      "msg": "Merkle proof is too deep"
    },
    {
      "code": 6033,
      "name": "invalidProof",
      "msg": "Merkle proof is invalid"
    },
    {
      "code": 6034,
      "name": "notLeafOwner",
      "msg": "Token account is not owned by the leaf owner"
    },
    {
      "code": 6035,
      "name": "wrongMint",
      "msg": "Token account is for the wrong mint"
    },
    {
      "code": 6036,
      "name": "holdWindowNotElapsed",
      "msg": "Hold window has not elapsed"
    },
    {
      "code": 6037,
      "name": "holdWindowElapsed",
      "msg": "Hold window has elapsed"
    },
    {
      "code": 6038,
      "name": "notTransferable",
      "msg": "Holder's mint account is not transferable yet"
    },
    {
      "code": 6039,
      "name": "mathOverflow",
      "msg": "Arithmetic overflow"
    },
    {
      "code": 6040,
      "name": "invariantViolated",
      "msg": "claimed + pending + swept would exceed total"
    },
    {
      "code": 6041,
      "name": "notHolderAta",
      "msg": "Holder's mint account must be their associated token account"
    }
  ],
  "types": [
    {
      "name": "attestor",
      "docs": [
        "[\"attestor\", issuer_config, attestor]. Closed by withdraw_bond."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "issuerConfig",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "bondAmount",
            "type": "u64"
          },
          {
            "name": "backend",
            "type": {
              "defined": {
                "name": "backend"
              }
            }
          },
          {
            "name": "activeCommitments",
            "type": "u32"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "backend",
      "docs": [
        "Reconstruction backend an attestor declares. Matches the manifest `backend` byte."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "ledgerReplay"
          },
          {
            "name": "stateArchive"
          }
        ]
      }
    },
    {
      "name": "challenge",
      "docs": [
        "[\"challenge\", distribution, index_u32_le]. Closed at resolution, rent to challenger (D1)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "distribution",
            "type": "pubkey"
          },
          {
            "name": "index",
            "type": "u32"
          },
          {
            "name": "challenger",
            "type": "pubkey"
          },
          {
            "name": "root",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "owner",
            "docs": [
              "Pubkey::default() disputes register_total (D22)."
            ],
            "type": "pubkey"
          },
          {
            "name": "claimedBalance",
            "type": "u64"
          },
          {
            "name": "bond",
            "type": "u64"
          },
          {
            "name": "openedSlot",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "claimReceipt",
      "docs": [
        "[\"claim\", distribution, owner]. Never closed; blocks a second claim."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "distribution",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "balance",
            "type": "u64"
          },
          {
            "name": "payout",
            "type": "u64"
          },
          {
            "name": "status",
            "type": {
              "defined": {
                "name": "claimStatus"
              }
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "claimStatus",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "paid"
          },
          {
            "name": "pending"
          }
        ]
      }
    },
    {
      "name": "commitArgs",
      "docs": [
        "Data an attestor commits (UC-4). Mirrors the manifest (spec section 11)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "root",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "manifestHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "registerTotal",
            "type": "u64"
          },
          {
            "name": "resolvedSlot",
            "type": "u64"
          },
          {
            "name": "firstCoveredSlot",
            "type": "u64"
          },
          {
            "name": "rows",
            "type": "u32"
          },
          {
            "name": "specVersion",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "commitment",
      "docs": [
        "[\"commitment\", distribution, attestor]. Closed at resolve or release_attestor (D8)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "distribution",
            "type": "pubkey"
          },
          {
            "name": "attestor",
            "type": "pubkey"
          },
          {
            "name": "root",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "manifestHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "backend",
            "type": {
              "defined": {
                "name": "backend"
              }
            }
          },
          {
            "name": "registerTotal",
            "type": "u64"
          },
          {
            "name": "resolvedSlot",
            "type": "u64"
          },
          {
            "name": "firstCoveredSlot",
            "type": "u64"
          },
          {
            "name": "rows",
            "type": "u32"
          },
          {
            "name": "specVersion",
            "type": "u16"
          },
          {
            "name": "postedSlot",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "disputeKind",
      "docs": [
        "Why a distribution is Disputed (D3): an owner challenge, or attestor roots that differ."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "owner"
          },
          {
            "name": "conflict"
          }
        ]
      }
    },
    {
      "name": "distribution",
      "docs": [
        "[\"distribution\", issuer_config, id_u64_le]. Never closed; Closed is terminal (D7).",
        "Owns the distribution vault."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "issuerConfig",
            "type": "pubkey"
          },
          {
            "name": "id",
            "type": "u64"
          },
          {
            "name": "paymentMint",
            "type": "pubkey"
          },
          {
            "name": "recordSlot",
            "type": "u64"
          },
          {
            "name": "total",
            "type": "u64"
          },
          {
            "name": "state",
            "type": {
              "defined": {
                "name": "distributionState"
              }
            }
          },
          {
            "name": "finalRoot",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "registerTotal",
            "type": "u64"
          },
          {
            "name": "saltSeedHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "claimedTotal",
            "type": "u64"
          },
          {
            "name": "pendingTotal",
            "type": "u64"
          },
          {
            "name": "sweptTotal",
            "type": "u64"
          },
          {
            "name": "declaredSlot",
            "type": "u64"
          },
          {
            "name": "windowEndSlot",
            "type": "u64"
          },
          {
            "name": "slotsRemaining",
            "type": "u64"
          },
          {
            "name": "openSlot",
            "type": "u64"
          },
          {
            "name": "commitmentCount",
            "type": "u32"
          },
          {
            "name": "challengeCount",
            "type": "u32"
          },
          {
            "name": "openChallengeIndex",
            "type": {
              "option": "u32"
            }
          },
          {
            "name": "disputeKind",
            "type": {
              "option": {
                "defined": {
                  "name": "disputeKind"
                }
              }
            }
          },
          {
            "name": "disputeOpenedSlot",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "distributionState",
      "docs": [
        "Distribution lifecycle. See the brief, section 4."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "declared"
          },
          {
            "name": "committed"
          },
          {
            "name": "disputed"
          },
          {
            "name": "open"
          },
          {
            "name": "closed"
          }
        ]
      }
    },
    {
      "name": "issuerConfig",
      "docs": [
        "[\"issuer\", mint]. Never closed (D18). Owns the bond vault."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "bondMint",
            "type": "pubkey"
          },
          {
            "name": "attestors",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "quorum",
            "type": "u8"
          },
          {
            "name": "requireBothBackends",
            "type": "bool"
          },
          {
            "name": "resolver",
            "type": "pubkey"
          },
          {
            "name": "noticeSlots",
            "type": "u64"
          },
          {
            "name": "finalityMarginSlots",
            "type": "u64"
          },
          {
            "name": "challengeWindowSlots",
            "type": "u64"
          },
          {
            "name": "resolveTimeoutSlots",
            "type": "u64"
          },
          {
            "name": "holdWindowSlots",
            "type": "u64"
          },
          {
            "name": "claimExpirySlots",
            "type": "u64"
          },
          {
            "name": "attestorBond",
            "type": "u64"
          },
          {
            "name": "challengerBond",
            "type": "u64"
          },
          {
            "name": "resolverFee",
            "type": "u64"
          },
          {
            "name": "distributionCount",
            "type": "u64"
          },
          {
            "name": "activeDistributions",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "issuerParams",
      "docs": [
        "Parameters set at init_issuer and replaced by update_issuer (D18, D19)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "attestors",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "quorum",
            "type": "u8"
          },
          {
            "name": "requireBothBackends",
            "type": "bool"
          },
          {
            "name": "resolver",
            "type": "pubkey"
          },
          {
            "name": "noticeSlots",
            "type": "u64"
          },
          {
            "name": "finalityMarginSlots",
            "type": "u64"
          },
          {
            "name": "challengeWindowSlots",
            "type": "u64"
          },
          {
            "name": "resolveTimeoutSlots",
            "type": "u64"
          },
          {
            "name": "holdWindowSlots",
            "type": "u64"
          },
          {
            "name": "claimExpirySlots",
            "type": "u64"
          },
          {
            "name": "attestorBond",
            "type": "u64"
          },
          {
            "name": "challengerBond",
            "type": "u64"
          },
          {
            "name": "resolverFee",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "pending",
      "docs": [
        "[\"pending\", distribution, owner]. A frozen holder's payout, held (D10, D11)."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "distribution",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "createdSlot",
            "type": "u64"
          },
          {
            "name": "payer",
            "type": "pubkey"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ruling",
      "docs": [
        "Resolver ruling for resolve_challenge (D3, D4, D5, D17)."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "upheld"
          },
          {
            "name": "rejected"
          },
          {
            "name": "conflict",
            "fields": [
              {
                "name": "root",
                "type": {
                  "array": [
                    "u8",
                    32
                  ]
                }
              }
            ]
          },
          {
            "name": "expired"
          }
        ]
      }
    }
  ],
  "constants": [
    {
      "name": "attestorSeed",
      "type": "bytes",
      "value": "[97, 116, 116, 101, 115, 116, 111, 114]"
    },
    {
      "name": "challengeSeed",
      "type": "bytes",
      "value": "[99, 104, 97, 108, 108, 101, 110, 103, 101]"
    },
    {
      "name": "claimSeed",
      "type": "bytes",
      "value": "[99, 108, 97, 105, 109]"
    },
    {
      "name": "commitmentSeed",
      "type": "bytes",
      "value": "[99, 111, 109, 109, 105, 116, 109, 101, 110, 116]"
    },
    {
      "name": "distributionSeed",
      "type": "bytes",
      "value": "[100, 105, 115, 116, 114, 105, 98, 117, 116, 105, 111, 110]"
    },
    {
      "name": "issuerSeed",
      "type": "bytes",
      "value": "[105, 115, 115, 117, 101, 114]"
    },
    {
      "name": "maxProofDepth",
      "docs": [
        "Merkle proofs are bounded before the loop (D13, spec section 8)."
      ],
      "type": "u8",
      "value": "32"
    },
    {
      "name": "pendingSeed",
      "type": "bytes",
      "value": "[112, 101, 110, 100, 105, 110, 103]"
    },
    {
      "name": "specVersion",
      "docs": [
        "docs/SNAPSHOT_SPEC.md v1.0."
      ],
      "type": "u16",
      "value": "100"
    }
  ]
};
