import { Link } from "react-router-dom"


function Welcome(){
 console.log(localStorage.getItem("profile"))
    return(
        <div style={{
                backgroundImage: "radial-gradient(circle, rgba(39, 39, 42, 1) 1.5px, transparent 1px)",
                backgroundSize: "20px 20px",
                backgroundRepeat: "repeat",
                }}
            className="flex flex-col justify-center items-center gap-1 h-[100vh] bg-[#3D3C3C] font-sans">
            <div className="flex felx-row justify-center text-6xl pb-5 font-bold">Pathfinder</div>
            <div className="felx flex-row justify-center text-center text-3xl">A tool to make your life easy</div>
            <Link to='/name'><button className="absolute bottom-3 right-5 flex flex-row justify-center py-1 px-3 bg-black rounded-md text-center text-white ">next</button></Link>
        </div>
    )
}
export default Welcome