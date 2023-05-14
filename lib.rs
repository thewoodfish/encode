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
        hash: Vec<u8>,
        name: Vec<u8>,
        party: Vec<u8>,
        votes: u64,
    }

    #[derive(scale::Decode, scale::Encode, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct Entry {
        name: Vec<u8>,
        time_frame: u64,
        candidates: Vec<Candidate>,
        bvns: Vec<Vec<u8>>,
    }

    #[ink(storage)]
    pub struct Encode {
        /// stores the election entries
        entries: Mapping<[u8; 32], Entry>,
    }

    impl Encode {
        /// Constructor that initializes the contract storage
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {
                entries: Default::default(),
            }
        }

        /// This message intitiliazes an election and sets the countdown
        #[ink(message, payable)]
        pub fn commence(
            &mut self,
            hash: [u8; 32],
            names: Vec<u8>,
            parties: Vec<u8>,
            blake_hashes: Vec<u8>,
            time_frame: u64,
            election_name: Vec<u8>,
        ) {
            // set up each candidate

            // set up the entry
            let mut entry = Entry {
                name: election_name,
                time_frame,
                candidates: Default::default(),
                bvns: Default::default(),
            };

            let mut parties = parties.split(|&c| c == b',');
            let mut bl_hashes = blake_hashes.split(|&c| c == b',');

            let _ = names
                .split(|&c| c == b',')
                .map(|n| {
                    let candidate = Candidate {
                        hash: bl_hashes.next().unwrap_or_default().to_vec(),
                        name: n.to_vec(),
                        party: parties.next().unwrap_or_default().to_vec(),
                        votes: 1, // because of JS parsing
                    };

                    entry.candidates.push(candidate);
                })
                .collect::<()>();

            self.entries.insert(hash, &entry);
        }

        /// This message returns all the candidates in the election
        #[ink(message, payable)]
        pub fn fetch_candidates(&self, hash: [u8; 32]) -> Vec<u8> {
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
                        collator.extend(c.hash.iter());
                        collator.extend([b'&', b'&']); // floor separator
                    })
                    .collect::<()>();

                // we'll be needing the name of the election, so we have to add it to what we'll return implicitly
                collator.extend([b'*', b'*', b'*']);    // our separator
                collator.extend(entry.name.iter());
                collator
            } else {
                Default::default()
            }
        }

        /// This message returns the time the election ends
        #[ink(message, payable)]
        pub fn fetch_time(&self, hash: [u8; 32]) -> u64 {
            if let Some(entry) = self.entries.get(&hash) {
                entry.time_frame
            } else {
                Default::default()
            }
        }

        // This message checks if the BVN is unique
        #[ink(message, payable)]
        pub fn bvn_isunique(&self, hash: [u8; 32], bvn: Vec<u8>) -> bool {
            if let Some(entry) = self.entries.get(&hash) {
                entry.bvns.contains(&bvn)
            } else {
                false
            }
        }

        // This message return the votes of the candidates
        #[ink(message, payable)]
        pub fn get_votes(&self, hash: [u8; 32]) -> Vec<u8> {
            if let Some(entry) = self.entries.get(&hash) {
                entry
                    .candidates
                    .iter()
                    .map(|c| c.votes.to_le_bytes())
                    .flatten()
                    .collect()
            } else {
                Default::default()
            }
        }

        /// This is the message that casts the vote on a voters behalf
        #[ink(message, payable)]
        pub fn cast_vote(&mut self, hash: [u8; 32], uniq_str: Vec<u8>, bvn: Vec<u8>) {
            let mut entry: Entry = Default::default();
            if let Some(e) = self.entries.get(&hash) {
                // make sure the user isn't voting more than once per BVN
                entry = e;
                if !entry.bvns.contains(&bvn) {
                    // increase vote count
                    for c in &mut entry.candidates {
                        if c.hash == uniq_str {
                            c.votes += 1;
                            break;
                        }
                    }
                    // store BVN for future purposes
                    entry.bvns.push(bvn);
                }
            }
            self.entries.insert(hash, &entry);
        }
    }
}
