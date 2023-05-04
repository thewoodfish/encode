#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod encode {
    use ink::storage::Mapping;
    use scale_info::prelude::vec::Vec;

    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Candidate {
        name: Vec<u8>,
        party: Vec<u8>,
        image_uri: Vec<u8>,
        votes: u64,
    }

    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct Entry {
        time_frame: u64,
        candidates: Vec<Candidate>,
    }

    #[ink(storage)]
    pub struct Encode {
        /// stores the election entries
        entries: Mapping<Vec<u8>, Entry>,
    }

    impl Encode {
        /// Constructor that initializes the contract storage
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                entries: Default::default(),
            }
        }

        /// This message intitiliazes an election and sets the countdown
        #[ink(message)]
        pub fn commence(
            &mut self,
            hash: Vec<u8>,
            names: Vec<u8>,
            parties: Vec<u8>,
            cids: Vec<u8>,
            time_frame: u64,
        ) {
            // set up each candidate

            // set up the entry
            let mut entry = Entry {
                time_frame,
                candidates: Default::default(),
            };

            let mut parties = parties.split(|&c| c == b',');
            let mut cids = cids.split(|&c| c == b',');

            let _ = names
                .split(|&c| c == b',')
                .map(|n| {
                    let candidate = Candidate {
                        name: n.to_vec(),
                        party: parties.next().unwrap_or_default().to_vec(),
                        image_uri: cids.next().unwrap_or_default().to_vec(),
                        votes: 0,
                    };

                    entry.candidates.push(candidate);
                })
                .collect::<()>();

            self.entries.insert(hash, &entry);
        }

        /// This message returns all the candidates in the election
        #[ink(message)]
        pub fn fetch_candidates(&self, hash: Vec<u8>) -> Vec<u8> {
            let mut collator = Vec::<u8>::new();
            if let Some(entry) = self.entries.get(&hash) {
                // we are going to fill up the collator byte by byte
                // and use distinguishing separators
                let _ = entry
                    .candidates
                    .iter()
                    .map(|c| {
                        collator.extend(c.name.iter());
                        collator.extend([b'%', b'%']);
                        collator.extend(c.party.iter());
                        collator.extend([b'%', b'%']);
                        collator.extend(c.image_uri.iter());
                        collator.extend([b'&', b'&']); // floor separator
                    })
                    .collect::<()>();
                collator
            } else {
                Default::default()
            }
        }

        /// This message returns the time the election ends
        #[ink(message)]
        pub fn fetch_time(&self, hash: Vec<u8>) -> u64 {
            if let Some(entry) = self.entries.get(&hash) {
                entry.time_frame
            } else {
                Default::default()
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new() {
            let mut e = Encode {
                entries: Default::default(),
            };

            let hash = "0xhjhs8s0d0sdsd8s0d90shcs09".as_bytes().to_vec();
            let parties = "Republican".as_bytes().to_vec();
            let names = "Donald Trump".as_bytes().to_vec();
            let cids = "".as_bytes().to_vec();
            let time = 9498283920;

            e.commence(hash.clone(), names, parties, cids, time);
            assert_eq!(e.fetch_time(hash), time);
        }
    }
}
