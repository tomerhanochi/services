CREATE TABLE file_server_role_grants (
    role TEXT NOT NULL,
    mount TEXT NOT NULL,
    permission TEXT NOT NULL CHECK (permission IN ('viewer', 'editor')),
    PRIMARY KEY (role, mount)
);
CREATE TABLE file_server_principal_roles (
    kind TEXT NOT NULL CHECK (kind IN ('user', 'machine')),
    principal_id TEXT NOT NULL,
    role TEXT NOT NULL,
    PRIMARY KEY (kind, principal_id, role)
);
