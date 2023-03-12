use dawclient::worker::AudioStreamingWorker;
use yew_agent::PublicWorker;

fn main() {
    AudioStreamingWorker::register();
}