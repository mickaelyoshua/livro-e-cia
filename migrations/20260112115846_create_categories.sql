CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_categories_updated_at
    BEFORE UPDATE ON categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

INSERT INTO categories (name, description) VALUES
    ('Bíblias', 'Bíblias em diversas traduções e formatos'),
    ('Estudos Bíblicos', 'Materiais para estudo aprofundado da Bíblia'),
    ('Devocionais', 'Livros devocionais para leitura diária'),
    ('Vida Cristã', 'Livros sobre vida cristã e crescimento espiritual'),
    ('Família e Relacionamentos', 'Livros sobre família, casamento e relacionamentos'),
    ('Infantil', 'Livros infantis e juvenis cristãos'),
    ('Teologia', 'Obras teológicas e acadêmicas'),
    ('Biografias', 'Biografias de líderes e missionários cristãos');
