CREATE TABLE categories (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name VARCHAR(100) NOT NULL UNIQUE,
	description TEXT
);

INSERT INTO categories (name, description) VALUES
	('Bíblias', 'Bíblias e edições de estudo'),
	('Estudos Bíblicos', 'Comentários e materiais de estudo'),
	('Devocionais', 'Leitura devocional diária'),
	('Vida Cristã', 'Crescimento e prática cristã'),
	('Família e Relacionamentos', 'Casamento, criação de filhos e vida familiar'),
	('Infantil', 'Livros para crianças'),
	('Teologia', 'Teologia e doutrina'),
	('Biografias', 'Histórias e testemunhos');
