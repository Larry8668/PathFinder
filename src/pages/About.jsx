import { Link } from "react-router-dom"


function About(){
 console.log(localStorage.getItem("name"))
 const username_json = localStorage.getItem("name")
 const username = JSON.parse(username_json);
    return(
        <div style={{
                backgroundImage: "radial-gradient(circle, rgba(39, 39, 42, 1) 1.5px, transparent 1px)",
                backgroundSize: "20px 20px",
                backgroundRepeat: "repeat",
                }}
            className="flex flex-col justify-center gap-5 p-5 h-[100vh] bg-[#3D3C3C] font-sans text-white">
            <div className="felx flex-row justify-left text-left text-4xl ">Hi {username.name}, Welcome to Pathfinder</div>
            <div className="text-left backdrop-blur-sm">
                A powerful Raycast-inspired launcher application built with Tauri and React. PathFinder provides instant access to your most-used tools and information through a beautiful, keyboard-driven interface. And you can access it just my pressing 
                </div>
            <div className="text-5xl text-center text-[#737373]">Ctrl+Shift+Space</div>
            <Link to='/ClipboardGuide'><button className="absolute bottom-3 left-180 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button></Link>
        </div>
    )
}
export default About