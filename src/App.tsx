
import './App.css'
import MyArc from './MyArc';
import * as eva from 'eva-icons';
import React from 'react';
import { invoke } from '@tauri-apps/api';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/api/notification';
import { clearInterval, clearTimeout, setInterval, setTimeout } from 'worker-timers';

// function App() {

class App extends React.Component {

  current_mode      = "not_started"
  time_sec_pomodoro = 50
  time_sec_current  = this.time_sec_pomodoro
  hook_stop_crono   = null
  pomodoro_name     = null
  muted             = false;

  constructor(props)
  {
    super(props);
    this.state = {
      progress: 1.0
    };
  }

  play_audio_click()
  {
    if (!this.muted)
    {
      const origAudio = document.getElementById("audio_click");
      const newAudio  = origAudio.cloneNode()
      // @ts-ignore
      newAudio.play()
    }
  }

  play_audio_notification()
  {
    if (!this.muted)
    {
      const origAudio = document.getElementById("audio_notification");
      const newAudio  = origAudio.cloneNode()
      // @ts-ignore
      newAudio.play()
    }
  }

  update_visuals()
  {
    console.log("time_sec_pomodoro", this.time_sec_pomodoro);
    console.log("time_sec_current", this.time_sec_current);
    // this.setState({progress:this.time_sec_current/this.time_sec_pomodoro})
    this.setState({progress:1.0})

    // @ts-ignore
    console.log(this.state.progress);

    if (this.current_mode == "not_started")
    {
      document.getElementById("container_repeat").style.display = "inline-block"
      document.getElementById("container_stop")  .style.display = "none"
      document.getElementById("container_play")  .style.display = "inline-block"
    }
    if (this.current_mode == "running")
    {
      document.getElementById("container_repeat").style.display = "none"
      document.getElementById("container_stop")  .style.display = "inline-block"
      document.getElementById("container_play")  .style.display = "none"
    }

    let a = Math.ceil(this.time_sec_pomodoro/ 60.0);
    document.getElementById("remaining_minutes").innerHTML = a.toString();
  }

  query_retrospective_pomodoro()
  {
    this.play_audio_click()
    document.getElementById("overlay_retrospective_pomodoro").style.display = "flex"
  }

  show_menu_config()
  {
    this.play_audio_click()
    document.getElementById("overlay_config").style.display = "flex"
  }

  query_cancel_pomodoro()
  {
    this.play_audio_click()
    document.getElementById("overlay_cancel_pomodoro").style.display = "flex"
  }

  mute()
  {
    this.muted = true;
    // // document.getElementById("icon_volume_on" ).style.display = "none"
    // // document.getElementById("icon_volume_off").style.display = "inline-block"
  }

  unmute()
  {
    this.muted = false;
    this.play_audio_click()
    // // document.getElementById("icon_volume_off").style.display = "none"
    // // document.getElementById("icon_volume_on" ).style.display = "inline-block"
  }

  sleep(ms)
  {
      return new Promise((resolve) => { setTimeout(resolve, ms); });
  }

  start_pomodoro()
  {
    console.log("start_pomodoro()");

    this.play_audio_click();

    // {

    // @ts-ignore
      let pomodoro_name = (document.getElementById("input_pomodoro_name")).value.trim()
      console.log(pomodoro_name)
      if (pomodoro_name == "" || pomodoro_name == null)
      {
        console.log("no input...")
        return ""
        // WIP show popup
      }
      
      this.current_mode = "running"
        
      this.update_visuals()

      invoke("pomodoro_start");

      let interval_timer = setInterval(() => {

        // console.log("...");

        if (this.time_sec_current > 0)
        {
          this.time_sec_current--;
          // @ts-ignore
          document.getElementById("remaining_minutes").innerHTML = Math.ceil(this.time_sec_current/ 60.0)
          this.setState({progress:this.time_sec_current/this.time_sec_pomodoro})
        }
        else
        {
          clearInterval(interval_timer)
          this.current_mode     = "not_started"
          this.time_sec_current = this.time_sec_pomodoro;
          this.setState({progress:1.0})
          this.update_visuals()
          this.play_audio_notification()
          sendNotification('🍅 Pomodoro finished!')
          invoke("annotate_pomodoro", {
            pomodoroName : pomodoro_name,
            durationInMin: Math.floor(this.time_sec_pomodoro / 60.0),
          }).then(to_create => {
            this.create_rows_left_panel(to_create)
          })
          invoke("pomodoro_end");
        }
      }, 1000)

      return interval_timer

    // }
    
  }

  close_pop_up()
  {
    this.play_audio_click()
    for (let x of document.getElementsByClassName("overlay_container"))
    {
      // @ts-ignore
      x.style.display = "none"
    }
  }

  cancel_pomodoro()
  {
    this.play_audio_click()

    this.close_pop_up()

    this.current_mode = "not_started"

    this.update_visuals()

    // arcProgress.updateProgress({progress:1.0})
    this.setState({progress:1.0})
    this.time_sec_current = this.time_sec_pomodoro

    if (this.hook_stop_crono != null)
    {
      clearInterval(this.hook_stop_crono)
      this.hook_stop_crono = null
    }

    // arcProgress.updateProgress({progress:1.0})
    this.setState({progress:1.0})

  }

  async start_retrospective_pomodoro_fill_untilnow()
  {
    let minutes_ago = 25;
    let last_date = await invoke("get_last_date_of_segment");
    if (last_date !== "")
    {
      // @ts-ignore
      minutes_ago = Math.floor(((new Date() - new Date(last_date)) / 1000 / 60) - 0.5);
    }

    this.close_pop_up()

    // @ts-ignore
    let pomodoro_name = (document.getElementById("input_pomodoro_name")).value.trim()
    if (pomodoro_name == "" || pomodoro_name == null)
    {
      return ""
      // WIP show popup
    }
    this.play_audio_click()
    this.current_mode = "not_started"
    this.update_visuals()
    this.time_sec_current = this.time_sec_pomodoro;
    // arcProgress.updateProgress({progress:1.0})
    this.setState({progress:1.0})

    invoke("annotate_pomodoro", {
      pomodoroName : pomodoro_name,
      durationInMin: minutes_ago  ,
    }).then(to_create => {
      this.create_rows_left_panel(to_create)
    })

    invoke("pomodoro_end");
  }

  start_retrospective_pomodoro_justnow()
  {
    this.close_pop_up()

    // @ts-ignore
    let pomodoro_name = (document.getElementById("input_pomodoro_name")).value
    if (pomodoro_name == "" || pomodoro_name == null)
    {
      return ""
      // WIP show popup
    }
    this.play_audio_click()
    this.current_mode = "not_started"
    this.update_visuals()
    this.time_sec_current = this.time_sec_pomodoro;
    // arcProgress.updateProgress({progress:1.0})
    this.setState({progress:1.0})

    invoke("annotate_pomodoro", {
      pomodoroName : pomodoro_name,
      durationInMin: Math.floor(this.time_sec_pomodoro / 60),
    }).then(to_create => {
      this.create_rows_left_panel(to_create)
    })

    invoke("pomodoro_end");
  }

  set_value_if_input(the_value)
  {
    this.play_audio_click();

    // @ts-ignore
    (document.getElementById("input_pomodoro_name")).value = the_value;
  }

  create_rows_left_panel(list_of_names)
  {

    // We remove the previously existing elements
    while (document.getElementById("list_of_events").firstChild)
    {
      document.getElementById("list_of_events").removeChild(document.getElementById("list_of_events").firstChild);
    }

    // // We-create them I
    // for (let x of list_of_names)
    // {
    //     var new_row = document.createElement("div");
    //     new_row.classList.add("row");
    //     new_row.classList.add("noselect");
    //     // @ts-ignore
    //     new_row.onclick   = (self) => { this.set_value_if_input(self.target.innerText); }
    //     // new_row.onclick   = (self) => { 
    //     //   // @ts-ignore
    //     //   this.set_value_if_input(self.target.innerText);
    //     //   // const i = self.target as HTMLElement;
    //     //   // this.set_value_if_input(i.innerText);
    //     // }
    //     new_row.innerHTML = x;
    //     document.getElementById("list_of_events").appendChild(new_row);
    // }




    // Given the variable list_of_names, we split each element by ">" and get the unique values each string starts with
    let list_of_roots = []
    for (let x of list_of_names)
    {
      let splitted = x.split(">")
      let to_add   = splitted[0].trim()

      // If is not in list_of_roots, we append it
      if (list_of_roots.indexOf(to_add) == -1)
      {
        list_of_roots.push(to_add)
      }
    }

    // We sort them in alphabetical order
    list_of_roots.sort()

    // Palettes n stuff (More here, https://coolors.co/palettes/popular/gradient)
    let palette_0 = ["#FF0000","#FF7F00","#FFFF00","#00FF00","#0000FF","#4B0082","#9400D3",]
    let palette_1 = ["#ADFFC7","#BBDCAD","#C8B993","#D6967A","#E47260","#F14F46","#FF2C2C"];
    let palette_2 = ["#8ecae6","#219ebc","#023047","#ffb703","#fb8500"];
    let palette_3 = ["#8ecae6","#219ebc","#f0f3bd","#bee3db"];
    let palette_4 = ["#004970","#0a9396","#ee9b00","#bb5903","#9b2226"];

    let palette = palette_4;
  
    // Colors are assigned to each root
    let i      = 0;
    let colors = {};
    for (let x of list_of_roots) {colors[x] = palette[i % palette.length];i++;}

    // We-create the roots
    for (let x of list_of_names)
    {
        var new_row = document.createElement("div");
        new_row.classList.add("row"     );
        new_row.classList.add("noselect");
        new_row.style.color = colors[x.split(">")[0].trim()]; // @ts-ignore
        new_row.onclick   = (self) => { this.set_value_if_input(self.target.innerText); }
        new_row.innerHTML = x;
        document.getElementById("list_of_events").appendChild(new_row);
    }

  }

  render()
  {

  return (
    <div className="App">

      <div className="overlay_container noselect" id='overlay_config' style={{"display":"none"}}>
        <div className="pop_up_dialog_box_with_gradient_borders" style={{"width":"80%"}}>

          <p id="overlay_text" style={{"alignSelf": "baseline"}}>Path to export long calendar data</p>
          <input id="input_path_to_export_lc" className="element_left" type="text" spellCheck="false"/>

          <div>
          <span id="icon_volume_on" onClick={() => this.mute()}>
            <i
            data-eva        = "volume-up-outline"
            data-eva-fill   = "#eee"
            data-eva-height = "48"
            data-eva-width  = "48"></i>
          </span>
          <span id="icon_volume_off" onClick={() => this.unmute()} style={{"display":"none"}}>
            <i
            data-eva        = "volume-off-outline"
            data-eva-fill   = "#eee"
            data-eva-height = "48"
            data-eva-width  = "48"></i>
          </span>
          <i
          onClick={() => this.close_pop_up()}
          data-eva        = "close-outline"
          data-eva-fill   = "#eee"
          data-eva-height = "48"
          data-eva-width  = "48"></i>
          </div>

          <div className="spacer_l"></div>

        </div>
      </div>

      <div className="overlay_container noselect" id="overlay_cancel_pomodoro">
      <div className="pop_up_dialog_box_with_gradient_borders">
        <p id="overlay_text">Are you sure you want to cancel the current pomodoro?</p>
        <div>
          <div className="button_a" onClick={() => this.cancel_pomodoro()}>Yes</div>
          <div style={{"display":"inline-block", "minWidth":"20px"}}></div>
          <div className="button_a" onClick={() => this.close_pop_up()}>No</div>
        </div>
      </div>
      </div>

      <div className="overlay_container noselect" id="overlay_retrospective_pomodoro">
        <div className="pop_up_dialog_box_with_gradient_borders">
          <p id="overlay_text">What kind of previous pomodoro do you want to annotate?</p>
          <div>
            <div className="button_a" onClick={() => this.start_retrospective_pomodoro_fill_untilnow()}>Fill until now</div>
            <div style={{"display":"inline-block","minWidth":"20px"}}></div>
            <div className="button_a" onClick={() => this.start_retrospective_pomodoro_justnow()}>Just add one now</div>
            <div style={{"display":"inline-block","minWidth":"20px"}}></div>
            <div className="button_a" onClick={() => this.close_pop_up()}>Cancel</div>
          </div>
        </div>
      </div>

      <div className="container_left">
        <input id="input_pomodoro_name" className="element_left" type="text" spellCheck="false"/>
        <div style={{"height":"20px"}}></div>
        <div id="list_of_events">
          asd ad
        </div>
      </div>

      <div className="container_right noselect">

      <div id="progress-container">
        {/* <ArcProgress
            // @ts-ignore
            progress   = {this.state.progress}
            size       = {300}
            speed      = {10}
            fillColor  = {'#eee'}
            emptyColor = {'#171717'}
            arcStart   = {0   + 90}
            arcEnd     = {360 + 90}
        /> */}

        <MyArc
          // @ts-ignore
          progress = {this.state.progress}
        />

      </div>

      <h1 id="remaining_minutes">--</h1>

      <div>
        <span id="container_repeat" onClick={() => {this.query_retrospective_pomodoro()}}>
          <i
          data-eva        = "arrow-circle-left-outline"
          data-eva-fill   = "#eee"
          data-eva-height = "48"
          data-eva-width  = "48"></i>
        </span>
        <span id="container_stop" onClick={() => {this.query_cancel_pomodoro()}}>
          <i
          data-eva        = "close-circle-outline"
          data-eva-fill   = "#eee"
          data-eva-height = "48"
          data-eva-width  = "48"></i>
        </span>
        <span id="container_play" onClick={() => {this.hook_stop_crono = this.start_pomodoro()}}>
          <i
          data-eva        = "play-circle-outline"
          data-eva-fill   = "#eee"
          data-eva-height = "48"
          data-eva-width  = "48"></i>
        </span>
      </div>

      </div>

      <audio id="audio_click"        src="./sound_click.wav"        preload="auto"></audio>
      <audio id="audio_notification" src="./sound_notification.mp3" preload="auto"></audio>

    </div>
  )

  }

  componentDidMount()
  {
    eva.replace();

    invoke('command_retrieve_last_pomodoros').then(to_create => {
      console.log(to_create);
      this.create_rows_left_panel(to_create)
    })

    invoke("conf_get_time_pomodoro_in_min", {}).then((time:number) => {
      this.time_sec_pomodoro = Math.floor(time * 60);
      this.time_sec_current  = this.time_sec_pomodoro;
      this.update_visuals();
      setTimeout(() => {this.update_visuals();}, 100); // Just in case...
    })

  }

}

export default App
