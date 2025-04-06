.PHONY: deploy

deploy:
	cd .. && \
	rsync -avzhe ssh --progress \
	--exclude 'target/' \
	--exclude '/.env' \
	--exclude 'assets/' \
	--exclude 'beast/target/' \
	--exclude 'beast/.env' \
	--exclude 'beast_common/target/' \
	--exclude 'beast_highscore_server/target/' \
	--exclude 'beast_highscore_server/.env' \
	--exclude 'beast_highscore_server/highscore.paw' \
	--exclude 'beast_highscore_server/highscores.ron' \
	./* deploy@dominik-wilkowski.com:/var/www/html/beast/
