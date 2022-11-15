import "dotenv/config";
import { initLogging, Manager, Player } from "../../";
import { Client, GatewayDispatchEvents } from "discord.js";

initLogging()

const client  = new Client({ intents: [ "Guilds", "GuildVoiceStates" ] });
const players = new Map<string, Player>();

client.on("ready", async () => {
    const manager = Manager.create({
        submitVoiceUpdate: (err, data) => {
            if (err) throw err;
            client.ws.shards.get(data.shardId)?.send({ op: 4, d: data.payload })
        },
        clientInfo: {
            userId: client.user.id,
            shardCount: client.ws.shards.size
        }
    });

    const player = new Player(manager, "323365823572082690");
    players.set(player.guildId, player);

    await player.join("381612756123648000");

    player.play();
});

client.on("debug", msg => {
    console.debug("[discord]", msg);
})

client.ws.on(GatewayDispatchEvents.VoiceServerUpdate, data => {
    const player = players.get(data.guild_id);
    if (player) player.updateVoiceServer(data);
});

client.ws.on(GatewayDispatchEvents.VoiceStateUpdate, data => {
    const player = players.get(data.guild_id);
    if (player) player.updateVoiceState(data);
});

client.login(process.env.DISCORD_TOKEN);
