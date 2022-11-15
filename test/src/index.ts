import "dotenv/config";
import { initLogging, Manager, Call, Input } from "../../";
import { Client, GatewayDispatchEvents } from "discord.js";

initLogging()

const client  = new Client({ intents: [ "Guilds", "GuildVoiceStates" ] });
const calls = new Map<string, Call>();

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

    const call = new Call(manager, "323365823572082690");
    calls.set(call.guildId, call);

    await call.join("381612756123648000");

    const input = Input.youtube(manager, "ytsearch:tory lanez - midnight interlude")
    console.log(input);

    const handle = call.play(input);
    handle.addEvent("end", async () => {
        const tc = client.channels.cache.get("1019030326745501747");
        if (tc.isTextBased()) tc.send("ended lol")
    });

});

client.on("debug", msg => {
    console.debug("[discord]", msg);
})

client.ws.on(GatewayDispatchEvents.VoiceServerUpdate, data => {
    const call = calls.get(data.guild_id);
    if (call) call.updateVoiceServer(data);
});

client.ws.on(GatewayDispatchEvents.VoiceStateUpdate, data => {
    const call = calls.get(data.guild_id);
    if (call) call.updateVoiceState(data);
});

client.login(process.env.DISCORD_TOKEN);
