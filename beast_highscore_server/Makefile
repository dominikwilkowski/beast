.PHONY: deploy

deploy:
	rsync -avzhe ssh --progress --exclude 'target/' --exclude '.env' --exclude 'highscore.paw' --exclude 'highscores.ron' ./* deploy@dominik-wilkowski.com:/var/www/html/beast/
