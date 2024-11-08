CREATE TABLE Ids (
    Id TEXT NOT NULL,
    Type TEXT NOT NULL,

    PRIMARY KEY (Id),
    FOREIGN KEY (Type) REFERENCES EntityTypes(Type)
);

CREATE TABLE EntityTypes (
    Type TEXT NOT NULL,

    PRIMARY KEY (Type)
);
