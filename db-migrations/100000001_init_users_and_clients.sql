INSERT INTO
    EntityTypes (Type)
VALUES
    ("User");

CREATE TABLE Users (
    Id TEXT NOT NULL,

    EmailAddress TEXT,
    DisplayName TEXT NOT NULL,

    PRIMARY KEY (Id),
    FOREIGN KEY (Id) REFERENCES Ids(Id),

    UNIQUE (DisplayName)
);
