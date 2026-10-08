-- Compte Echo : le serveur ne garde rien de lisible. La cle d'acces est derivee du mot de passe sur la machine de
-- l'utilisateur ; le serveur n'en garde qu'une empreinte. Le coffre contient des donnees chiffrees de bout en bout.
CREATE TABLE IF NOT EXISTS comptes (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  empreinte_acces TEXT NOT NULL,
  sel_serveur TEXT NOT NULL,
  sel_client TEXT NOT NULL,
  iterations INTEGER NOT NULL,
  cree_le INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (
  empreinte_jeton TEXT PRIMARY KEY,
  compte TEXT NOT NULL,
  expire_le INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS coffre (
  compte TEXT NOT NULL,
  type TEXT NOT NULL,
  version INTEGER NOT NULL,
  donnees TEXT NOT NULL,
  maj_le INTEGER NOT NULL,
  PRIMARY KEY (compte, type)
);
CREATE TABLE IF NOT EXISTS echecs (
  email TEXT NOT NULL,
  le INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS echecs_email ON echecs (email, le);
-- Tableau de bord : derniere activite de chaque compte et version d'Echo qui l'a faite (rien du contenu).
CREATE TABLE IF NOT EXISTS activite (
  compte TEXT PRIMARY KEY,
  vu_le INTEGER NOT NULL,
  version TEXT
);
-- Compteurs par jour (AAAA-MM-JJ) : requetes par route et statut.
CREATE TABLE IF NOT EXISTS compteurs (
  jour TEXT NOT NULL,
  cle TEXT NOT NULL,
  n INTEGER NOT NULL,
  PRIMARY KEY (jour, cle)
);
-- Comptes administrateurs : la section Administration d'Echo s'ouvre pour eux (le serveur verifie a chaque appel).
CREATE TABLE IF NOT EXISTS admins (
  compte TEXT PRIMARY KEY
);
