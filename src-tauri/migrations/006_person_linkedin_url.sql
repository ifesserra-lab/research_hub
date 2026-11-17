-- SEP-040: Adiciona a 22ª coluna `linkedin_url` nas entidades de pessoas
-- (researchers e students) tanto no Horizon Legacy quanto no novo Horizon (hv2_*),
-- alinhando o schema à versão final do pacote `exports_canonical.zip`.

ALTER TABLE researchers ADD COLUMN linkedin_url TEXT;
ALTER TABLE students ADD COLUMN linkedin_url TEXT;
ALTER TABLE hv2_researchers ADD COLUMN linkedin_url TEXT;
ALTER TABLE hv2_students ADD COLUMN linkedin_url TEXT;
