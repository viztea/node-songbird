import "dotenv/config";
import {Call, initLogging, Input, Manager, PlayModeValue, ReqwestClient, TrackHandle, TrackHandleEvent} from "../../";
import {Client, GatewayDispatchEvents} from "discord.js";
import {extractID, InfoData, search, video_info} from "play-dl";
import dayjs from "dayjs";
import duration from "dayjs/plugin/duration";
import parseDuration from "parse-duration";
import EventEmitter from "events";

dayjs.extend(duration)

initLogging()

interface Track {
    readonly handle: TrackHandle;
    readonly info: InfoData;
}

const client = new Client({intents: ["Guilds", "GuildVoiceStates", "GuildMessages", "MessageContent"]});
let manager: Manager;

class Player extends EventEmitter {
    readonly call: Call;

    track?: Track;

    constructor(public readonly guildId: string) {
        super();

        this.call = new Call(manager, guildId);
    }

    async play(query: string) {
        /* extract video id from query. */
        let videoId: string
        if (/^https?:\/\//.test(query)) {
            videoId = extractID(query)
        } else {
            const results = await search(query, {source: {youtube: "video"}});
            videoId = results[0].id;
        }

        /* get video formats */
        const video: InfoData = await video_info(videoId);

        const format = video.format
            .sort((a, b) => b.bitrate! - a.bitrate!)
            .filter(a => a.mimeType?.includes("opus"))
            [0];

        /* play the url given to use by the format. */
        const input = Input.http(
            reqwest,
            format.url
        );

        const handle = this.call.play(input);
        this.track = {
            handle,
            info: video
        }

        handle.addEvent(TrackHandleEvent.Playable, () => {
            if (this.track) {
                return;
            }

            this.emit("start", this.track);
        });

        handle.addEvent(TrackHandleEvent.End, () => {
            this.emit("end", this.track);
            delete this.track;
        });

        handle.addEvent(TrackHandleEvent.Error, async () => {
            const status = await handle.getInfo();
            if (status.playing.value == PlayModeValue.Errored) {
                this.emit("error", this.track, new Error(status.playing.error));
            }

            delete this.track;
        });
    }
}

const players = new Map<string, Player>(), reqwest = new ReqwestClient();

client.on("ready", async () => {
    manager = Manager.create({
        userId: client.user.id,
        driver: {
            submitVoiceUpdate: (err, data) => {
                if (err) throw err;
                client.ws.shards.get(data.shardId)?.send({op: 4, d: data.payload})
            },
            shardCount: client.ws.shards.size
        }
    });

    // const call = new Call(manager, "323365823572082690");
    // calls.set(call.guildId, call);
    //
    // await call.join("381612756123648000");
    //
    // const input = Input.youtube(manager, "ytsearch:tory lanez - midnight interlude")
    // console.log(input);
    //
    // const handle = call.play(input);
    //
    // let playing = false;
    // handle.addEvent(TrackHandleEvent.Playable, async () => {
    //     if (playing) return;
    //     playing = true;
    //
    //     console.log("playable");
    //     await new Promise(res => setTimeout(res, 5000))
    //     console.log("bruh");
    //     await handle.seek(6e4);
    // });
    //
    // handle.addEvent(TrackHandleEvent.End, async () => {
    //     const tc = await client.channels.fetch("1016281738772750407");
    //     if (tc.isTextBased()) tc.send("ended lol")
    // });
});

client.on("messageCreate", async message => {
    if (message.author.bot || !message.guild || !message.content.startsWith("!")) return;

    try {
        const [command, ...args] = message.content.slice(1).split(" ");
        switch (command.toLowerCase()) {
            case "join": {
                if (players.has(message.guildId)) return void message.reply("A okater already exists for this guild.");

                const memberVoiceChannel = message.member?.voice?.channelId
                if (!memberVoiceChannel) return void message.reply("join vc you idiot");

                const player = new Player(message.guildId);
                players.set(message.guildId, player);

                player.on("start", track => {
                    message.channel.send(`**Now Playing:** ${track.info.video_details.title}`);
                });

                player.on("end", track => {
                    message.channel.send(`**Stopped Playing:** ${track.info.video_details.title}`);
                });

                player.on("error", (track, error) => {
                    message.channel.send(`${track.info.video_details.title} encountered error:\n\`\`\`js\n${error}\n\`\`\``);
                });

                await player.call.join(memberVoiceChannel);
                return void message.reply(`ok, im joining <#${memberVoiceChannel}>`);
            }

            case "play": {
                const player = players.get(message.guildId);
                if (!player) return void message.reply("A call does not exist for this guild.");
                return void player.play(args.join(" "));
            }

            case "play_old": {
                const player = players.get(message.guildId);
                if (!player) return void message.reply("A call does not exist for this guild.");

                const _query = args.join(" ");

                /* play the url given to use by the format. */
                const input = Input.youtube(
                    manager,
                    /^https?:\/\//.test(_query) ? _query : "ytsearch:" + _query
                );

                return void player.call.play(input);
            }

            case "np":
            case "nowplaying": {
                const player = players.get(message.guildId);
                if (!player) return void message.reply("A player does not exist for this guild.");

                const track = player.track
                if (!track) return void message.reply("Nothing is currently playing.")

                const info = await track.handle.getInfo();

                return void message.reply(`${track.info.video_details.title} @ ${dayjs.duration(info.position).format("mm:ss")}`)
            }

            case "seek": {
                const player = players.get(message.guildId);
                if (!player) return void message.reply("A player does not exist for this guild.");


                const track = player.track
                if (!track) return void message.reply("Nothing is currently playing.")

                const input = args.join(" "), timecode = parseDuration(input);
                console.log(await player.track.handle.seek(timecode));

                return void message.reply(`Seeking to ${dayjs.duration(timecode).format("mm:ss")}`);
            }

            case "stop": {
                const player = players.get(message.guildId);
                if (!player) return void message.reply("A player does not exist for this guild.");

                return void player.call.stop()
            }
        }
    } catch (ex) {
        message.reply(`Command ran into an exception: \`\`\`js\n${ex}\n\`\`\``);
    }
})

client.on("debug", msg => {
    console.debug("[discord]", msg);
})

client.ws.on(GatewayDispatchEvents.VoiceServerUpdate, data => {
    const player = players.get(data.guild_id);
    if (player) player.call.updateVoiceServer(data);
});

client.ws.on(GatewayDispatchEvents.VoiceStateUpdate, data => {
    if (data.user_id !== client.user.id) return;

    const player = players.get(data.guild_id);
    if (player) player.call.updateVoiceState(data);
});

client.login(process.env.DISCORD_TOKEN);
