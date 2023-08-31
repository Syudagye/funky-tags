CREATE TABLE Lad(
    id INTEGER PRIMARY KEY NOT NULL,
    username VARCHAR(255) NOT NULL,
    passwd_hash CHAR(64) NOT NULL
);

CREATE TABLE Game(
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE Gamertag(
    id INTEGER PRIMARY KEY NOT NULL,
    username VARCHAR(255) NOT NULL,
    poster INTEGER NOT NULL,
    game INTEGER NOT NULL,
    FOREIGN KEY (poster) REFERENCES Lad(id),
    FOREIGN KEY (game) REFERENCES Game(id)
);
