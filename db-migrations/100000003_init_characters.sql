INSERT INTO
    EntityTypes (Type)
VALUES
    ("Character");

CREATE TABLE Characters (
    Id TEXT NOT NULL,
    UserId TEXT NOT NULL,

    Name TEXT,
    NickName TEXT NOT NULL,
    Race TEXT NOT NULL CHECK (Race IN ("Android", "Cyborg", "Human")),

    PRIMARY KEY (Id),
    FOREIGN KEY (Id) REFERENCES Ids(Id),

    FOREIGN KEY (UserId) REFERENCES Users(Id),
    UNIQUE (UserId, Nickname)
);

CREATE TABLE Humans (
    Id TEXT NOT NULL,

    PRIMARY KEY (Id),
    FOREIGN KEY (Id) REFERENCES Characters(Id)
);

CREATE TABLE Androids (
    Id TEXT NOT NULL,

    PRIMARY KEY (Id),
    FOREIGN KEY (Id) REFERENCES Characters(Id)
);

CREATE TABLE Cyborgs (
    Id TEXT NOT NULL,

    PRIMARY KEY (Id),
    FOREIGN KEY (Id) REFERENCES Characters(Id)
);
