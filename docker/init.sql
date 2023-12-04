CREATE TABLE found_blocks (
                              id INT AUTO_INCREMENT PRIMARY KEY,
                              puid INT,
                              worker_id BIGINT,
                              worker_full_name VARCHAR(255),
                              height VARCHAR(255),
                              hash VARCHAR(255),
                              hash_no_nonce VARCHAR(255),
                              nonce VARCHAR(255),
                              prev_hash VARCHAR(255),
                              network_diff INT,
                              created_at TIMESTAMP
);